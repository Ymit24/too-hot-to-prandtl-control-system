#![no_std]
#![no_main]

use arduino_mkrzero as bsp;
use bsp::hal;

use panic_halt as _;

use hal::clock::GenericClockController;
use hal::prelude::*;

use rtic::app;
use usbd_serial::SerialPort;

#[app(device = bsp::pac, peripherals=true, dispatchers=[EVSYS])]
mod app {
    use super::*;
    use common::packet::{ReportControlTargetsPacket, ReportLogLinePacket, *};
    use cortex_m::peripheral::NVIC;
    use fixedstr::str64;
    use hal::pac::interrupt;
    use hal::{
        clock::{ClockGenId, ClockSource},
        pac::Interrupt,
        rtc::Count32Mode,
        usb::usb_device::class_prelude::UsbBusAllocator,
        usb::UsbBus,
    };
    use heapless::spsc::{Consumer, Producer, Queue};
    use usb_device::device::{UsbDevice, UsbDeviceBuilder, UsbVidPid};
    use usb_device::UsbError;
    use usbd_serial::{SerialPort, USB_CLASS_CDC};
    #[shared]
    struct Shared {
        device: UsbDevice<'static, UsbBus>,
        serial: SerialPort<'static, UsbBus>,
    }

    #[local]
    struct Local {
        tx_packets: Producer<'static, Packet, 16>, // TODO: MIGHT CHANGE TO JUST THE BUF
        rx_packets: Consumer<'static, Packet, 16>,

        led: bsp::pins::Led,

        led_commands_producer: Producer<'static, bool, 16>,
        led_commands_consumer: Consumer<'static, bool, 16>,
    }

    #[monotonic(binds = RTC, default = true)]
    type RtcMonotonic = hal::rtc::Rtc<Count32Mode>;

    #[init(local=[
           usb_bus: Option<UsbBusAllocator<UsbBus>> = None,
           q: Queue<Packet,
           16> = Queue::new(),
           led_commands_queue: Queue<bool, 16> = Queue::new()
    ])]
    fn init(cx: init::Context) -> (Shared, Local, init::Monotonics) {
        let mut peripherals = cx.device;
        let pins = bsp::pins::Pins::new(peripherals.PORT);
        let mut core = cx.core;
        let mut clocks = GenericClockController::with_external_32kosc(
            peripherals.GCLK,
            &mut peripherals.PM,
            &mut peripherals.SYSCTRL,
            &mut peripherals.NVMCTRL,
        );

        let _gclk = clocks.gclk0();
        let rtc_clock_src = clocks
            .configure_gclk_divider_and_source(ClockGenId::GCLK2, 1, ClockSource::XOSC32K, false)
            .unwrap();
        clocks.configure_standby(ClockGenId::GCLK2, true);
        let rtc_clock = clocks.rtc(&rtc_clock_src).unwrap();
        let rtc =
            hal::rtc::Rtc::count32_mode(peripherals.RTC, rtc_clock.freq(), &mut peripherals.PM);

        let led = bsp::pin_alias!(pins.led).into();
        let usb_n = bsp::pin_alias!(pins.usb_n);
        let usb_p = bsp::pin_alias!(pins.usb_p);

        let usb_bus: &'static _ = cx.local.usb_bus.insert(bsp::usb::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            usb_n.into(),
            usb_p.into(),
        ));

        let serial_port = SerialPort::new(usb_bus);
        let device = UsbDeviceBuilder::new(usb_bus, UsbVidPid(0x2222, 0x3333))
            .manufacturer("LA Tech")
            .product("Too Hot To Prandtl Controller")
            .serial_number("1324")
            .device_class(USB_CLASS_CDC)
            .build();

        unsafe {
            core.NVIC.set_priority(interrupt::USB, 1);
            NVIC::unmask(interrupt::USB);
        }

        core.SCB.set_sleepdeep();

        let (tx_packets, rx_packets) = cx.local.q.split();
        let (led_commands_producer, led_commands_consumer) = cx.local.led_commands_queue.split();

        blink::spawn().unwrap();
        //send_packets::spawn().unwrap();
        task_led_commander::spawn().unwrap();
        task_usb_io::spawn().unwrap();

        (
            Shared {
                device,
                serial: serial_port,
            },
            Local {
                tx_packets,
                rx_packets,
                led_commands_consumer,
                led_commands_producer,
                led,
            },
            init::Monotonics(rtc),
        )
    }

    #[task(local=[tx_packets])]
    fn blink(mut cx: blink::Context) {
        let pack = Packet::ReportSensors(ReportSensorsPacket {
            fan_speed_norm: 100,
            pump_speed_norm: 200,
            valve_state: ValveState::Opening,
        });

        let tx_packets = cx.local.tx_packets;
        if tx_packets.ready() {
            let _ = tx_packets.enqueue(pack.clone());
        }

        //       let pack = Packet::ReportLogLine(ReportLogLinePacket {
        //           log_line: str64::from("abc"),
        //       });
        //       if tx_packets.ready() {
        //           tx_packets.enqueue(pack.clone()).unwrap(); // NOTE: Should always be good.
        //       }
        blink::spawn_after(hal::rtc::Duration::secs(1u32)).ok();
    }

    #[task(local=[led_commands_consumer])]
    fn task_led_commander(mut cx: task_led_commander::Context) {
        let consumer = cx.local.led_commands_consumer;
        // let led = cx.local.led;

        if consumer.ready() {
            while let Some(command) = consumer.dequeue() {
                if command {
                    //led.set_high();
                } else {
                    //led.set_low();
                }
            }
        }

        task_led_commander::spawn_after(hal::rtc::Duration::millis(100)).ok();
    }

    #[task(shared=[serial], local=[led_commands_producer, rx_packets,led])]
    fn task_usb_io(mut cx: task_usb_io::Context) {
        let mut serial = cx.shared.serial;
        let mut tx_led_commands = cx.local.led_commands_producer;
        let led = cx.local.led;

        let mut buf = [0u8; 128];
        let bytes = serial.lock(|serial_locked| match serial_locked.read(&mut buf) {
            Err(e) => 0,
            Ok(bytes_read) => bytes_read,
        });
        if bytes != 0 {
            decode_and_process_packets(&buf[0..bytes], &mut tx_led_commands, led);
        }

        let rx_packets = cx.local.rx_packets;
        while let Some(packet) = rx_packets.dequeue() {
            let buffer: heapless::Vec<u8, 128> = postcard::to_vec(&packet).unwrap();
            serial.lock(|serial_locked| {
                let _ = serial_locked.write(&buffer);
            });
        }
        serial.lock(|serial_locked| {
            let _ = serial_locked.flush();
        });

        task_usb_io::spawn_after(hal::rtc::Duration::millis(500)).ok();
    }

    fn decode_and_process_packets(
        buffer: &[u8],
        tx_led_commands: &mut Producer<bool, 16>,
        led: &mut bsp::pins::Led,
    ) {
        let mut remaining = buffer;
        while let Ok((packet, other)) = postcard::take_from_bytes::<Packet>(remaining) {
            remaining = other;
            match packet {
                Packet::ReportControlTargets(packet) => {
                    if packet.command {
                        led.set_high();
                    } else {
                        led.set_low();
                    }
                }
                _ => {}
            }
        }
    }

    #[task(binds=USB, shared=[serial, device])]
    fn usb(cx: usb::Context) {
        let device = cx.shared.device;
        let serial = cx.shared.serial;

        // NOTE: Change this to always be able to produce bytes without lock maybe?
        (device, serial).lock(|device_locked, serial_locked| {
            device_locked.poll(&mut [serial_locked]);
        });
    }
}
