#![no_std]
#![no_main]

use arduino_mkrzero as bsp;
use bsp::hal;
use bsp::pins::Led;
use common::packet::Packet;
use cortex_m::peripheral::NVIC;
use embedded_firmware_core::Application;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::{OutputPin, ToggleableOutputPin};
use panic_halt as _;

use bsp::entry;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals};
use hal::usb::UsbBus;

use usb_device::bus::UsbBusAllocator;

static mut BUS_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut APPLICATION: Option<Application<'static, UsbBus, Delay, Led>> = None;

fn initialize() {
    let mut peripherals = Peripherals::take().unwrap();
    let mut core = CorePeripherals::take().unwrap();
    let mut clocks = GenericClockController::with_external_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );
    let pins = bsp::pins::Pins::new(peripherals.PORT);
    let mut led = bsp::pin_alias!(pins.led).into_push_pull_output();
    let mut delay = Delay::new(core.SYST, &mut clocks);

    let usb_n = bsp::pin_alias!(pins.usb_n);
    let usb_p = bsp::pin_alias!(pins.usb_p);

    // this stays
    unsafe {
        BUS_ALLOCATOR = Some(bsp::usb::usb_allocator(
            peripherals.USB,
            &mut clocks,
            &mut peripherals.PM,
            usb_n.into(),
            usb_p.into(),
        ));
    }

    // NOTE: This must happen before we enable USB interrupt.
    unsafe {
        APPLICATION = Some(Application::new(
            BUS_ALLOCATOR.as_ref().unwrap(),
            delay,
            led,
        ));
    }

    // this stays
    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }
}

// TODO: Finish feature parity with RTIC version.
// [ ] PWM Pump/Fan control
// [ ] ADC Pump/Fan input
// [ ] Valve digital output
// [ ] Valve digital input (determine valve state)

#[entry]
fn main() -> ! {
    initialize();

    let app = unsafe { APPLICATION.as_mut().unwrap() };

    // NOTE: DEBUG CODE
    let mut counter = 0;

    loop {
        cortex_m::interrupt::free(|cs| unsafe {
            app.read_packets_from_usb(cs);
            app.write_packets_to_usb(cs);
        });

        // NOTE: DEBUG CODE
        counter += 1;
        if counter >= 4 {
            counter -= 4;

            // NOTE: DEBUG CODE
            while let Some(packet) = app.incoming_packets.pop() {
                match packet {
                    Packet::ReportControlTargets(control_packet) => {
                        app.led.set_state(control_packet.command.into());
                    }
                    _ => {}
                }
            }

            // NOTE: DEBUG CODE
            for i in 0..2 {
                app.outgoing_packets.push(Packet::ReportSensors(
                    common::packet::ReportSensorsPacket {
                        fan_speed_norm: 100,
                        pump_speed_norm: 200,
                        valve_state: common::packet::ValveState::Open,
                    },
                ));
            }
        }

        app.delay.delay_ms(100u16);
    }
}

#[interrupt]
fn USB() {
    unsafe {
        APPLICATION.as_mut().unwrap().poll_usb();
    }
}
