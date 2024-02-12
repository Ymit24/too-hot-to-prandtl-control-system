#![no_std]
#![no_main]

use arduino_mkrzero as bsp;
use bsp::hal;

use panic_halt as _;

use hal::clock::GenericClockController;
use hal::prelude::*;

use rtic::app;
use usbd_serial::SerialPort;

mod blink;
use blink::blink_internal;

mod led_commander;
use led_commander::led_commander_internal;

mod usb_io;
use usb_io::task_usb_io_internal;

mod write_control_targets_out;
use write_control_targets_out::*;

#[app(device = bsp::pac, peripherals=true, dispatchers=[EVSYS])]
mod app {
    use super::*;
    use common::packet::{ReportControlTargetsPacket, ReportLogLinePacket, *};
    use cortex_m::peripheral::NVIC;
    use fixedstr::str64;
    use hal::pac::{gclk, interrupt};
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

        rx_control_frames: Consumer<'static, ReportControlTargetsPacket, 4>,
        tx_control_frames: Producer<'static, ReportControlTargetsPacket, 4>,
    }

    #[monotonic(binds = RTC, default = true)]
    type RtcMonotonic = hal::rtc::Rtc<Count32Mode>;

    #[init(local=[
           usb_bus: Option<UsbBusAllocator<UsbBus>> = None,
           q: Queue<Packet,
           16> = Queue::new(),
           led_commands_queue: Queue<bool, 16> = Queue::new(),
           control_frame_queue: Queue<ReportControlTargetsPacket, 4> = Queue::new()
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

        let gclk = clocks.gclk0();
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
        let (tx_control_frames, rx_control_frames) = cx.local.control_frame_queue.split();

        let _a4 = pins.pa04.into_mode::<hal::gpio::AlternateE>();
        let tcc0_tcc1_clock: &hal::clock::Tcc0Tcc1Clock = &clocks.tcc0_tcc1(&gclk).unwrap();
        let mut pwm0 = hal::pwm::Pwm0::new(
            &tcc0_tcc1_clock,
            1u32.kHz(),
            peripherals.TCC0,
            &mut peripherals.PM,
        );

        let max_duty_cycle = pwm0.get_max_duty();
        pwm0.enable(hal::pwm::Channel::_0);
        pwm0.set_duty(hal::pwm::Channel::_0, max_duty_cycle);

        blink::spawn().unwrap();
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
                tx_control_frames,
                rx_control_frames,
            },
            init::Monotonics(rtc),
        )
    }

    #[task(local=[tx_packets])]
    fn blink(mut cx: blink::Context) {
        let tx_packets = cx.local.tx_packets;
        blink_internal(tx_packets);
        blink::spawn_after(hal::rtc::Duration::secs(1u32)).ok();
    }

    #[task(local=[led_commands_consumer,led])]
    fn task_led_commander(mut cx: task_led_commander::Context) {
        led_commander_internal(cx);

        task_led_commander::spawn_after(hal::rtc::Duration::millis(100)).ok();
    }

    #[task(local=[rx_control_frames])]
    fn task_write_control_targets_out(cx: task_write_control_targets_out::Context) {
        task_write_control_targets_out_internal();

        task_write_control_targets_out::spawn_after(hal::rtc::Duration::millis(500)).ok();
    }

    #[task(shared=[serial], local=[led_commands_producer, rx_packets])]
    fn task_usb_io(mut cx: task_usb_io::Context) {
        task_usb_io_internal(cx);

        task_usb_io::spawn_after(hal::rtc::Duration::millis(500)).ok();
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
