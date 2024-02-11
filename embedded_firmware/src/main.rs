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
    use cortex_m::peripheral::NVIC;
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
        p: Producer<'static, Packet<'static>, 5>, // TODO: MIGHT CHANGE TO JUST THE BUF
        c: Consumer<'static, Packet<'static>, 5>,

        led: bsp::pins::Led,

        led_commands_producer: Producer<'static, bool, 10>,
        led_commands_consumer: Consumer<'static, bool, 10>,
    }

    #[monotonic(binds = RTC, default = true)]
    type RtcMonotonic = hal::rtc::Rtc<Count32Mode>;

    #[init(local=[usb_bus: Option<UsbBusAllocator<UsbBus>> = None, q: Queue<Packet<'static>, 5> = Queue::new(),led_commands_queue: Queue<bool, 10> = Queue::new()])]
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

        let (p, c) = cx.local.q.split();
        let (led_commands_producer, led_commands_consumer) = cx.local.led_commands_queue.split();

        blink::spawn().unwrap();
        send_packets::spawn().unwrap();
        task_led_commander::spawn().unwrap();

        (
            Shared {
                device,
                serial: serial_port,
            },
            Local {
                p,
                c,
                led_commands_consumer,
                led_commands_producer,
                led,
            },
            init::Monotonics(rtc),
        )
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Clone)]
    struct Packet<'a> {
        type_id: u8,
        data: &'a str,
        command: bool,
    }

    #[task(shared=[serial], local=[c])]
    fn send_packets(mut cx: send_packets::Context) {
        let c = cx.local.c;
        let mut serial = cx.shared.serial;
        if let Some(packet) = c.dequeue() {
            let bytes: heapless::Vec<u8, 64> = postcard::to_vec(&packet).unwrap();

            serial.lock(|serial_locked| {
                let _ = serial_locked.write(&bytes);
                let _ = serial_locked.flush();
            });
        } else {
            // nothing was ready
        }

        send_packets::spawn_after(hal::rtc::Duration::millis(500u32)).ok();
    }

    #[task(local=[p])]
    fn blink(mut cx: blink::Context) {
        let pack = Packet {
            type_id: 8,
            data: "__Hello World__",
            command: false,
        };

        let p = cx.local.p;
        if p.ready() {
            p.enqueue(pack.clone()).unwrap(); // NOTE: Should always be good.
        }

        blink::spawn_after(hal::rtc::Duration::secs(1u32)).ok();
    }

    #[task(local=[led_commands_consumer,led])]
    fn task_led_commander(mut cx: task_led_commander::Context) {
        let consumer = cx.local.led_commands_consumer;
        let led = cx.local.led;

        if consumer.ready() {
            while let Some(command) = consumer.dequeue() {
                if command {
                    led.set_high();
                } else {
                    led.set_low();
                }
            }
        }

        task_led_commander::spawn_after(hal::rtc::Duration::millis(100)).ok();
    }

    #[task(binds=USB, shared=[serial, device], local=[led_commands_producer])]
    fn usb(cx: usb::Context) {
        let device = cx.shared.device;
        let serial = cx.shared.serial;
        let led_commands_producer = cx.local.led_commands_producer;
        (device, serial).lock(|device_locked, serial_locked| {
            if device_locked.poll(&mut [serial_locked]) {
                let mut buf = [0u8; 64];
                match serial_locked.read(&mut buf) {
                    Err(_e) => {
                        //          let _ = serial_locked.write(b"Did Receive ERROR.\n");
                        //          let _ = serial_locked.flush();
                    }
                    Ok(0) => {
                        //                        let _ = serial_locked.write(b"Didn't receive data.\n");
                        //                        let _ = serial_locked.flush();
                    }
                    Ok(_count) => {
                        let mut remaining: &[u8] = &buf[0.._count];

                        let mut _cmd: bool = false;
                        while let Ok((packet, other)) =
                            postcard::take_from_bytes::<Packet>(remaining)
                        {
                            // process packets
                            remaining = other;
                            if led_commands_producer.ready() {
                                led_commands_producer.enqueue(packet.command).unwrap();
                            }
                        }
                    }
                }
            }
        });
    }
}
