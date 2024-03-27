#![no_std]
#![no_main]

use arduino_mkrzero as bsp;
use bsp::hal;
use bsp::pins::Led;
use common::packet::Packet;
use cortex_m::peripheral::NVIC;
use embedded_firmware_core::application::Application;
use embedded_firmware_core::PrandtlAdc;
use embedded_hal::adc::Channel as AdcChannel;
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::digital::v2::OutputPin;
use hal::adc::Adc;
use hal::gpio::{Alternate, Pin, B, PA04, PA05, PA06, PA07};
use hal::pwm::{Channel, Pwm0, Pwm1};
use panic_halt as _;

use bsp::entry;
use hal::clock::GenericClockController;
use hal::delay::Delay;
use hal::pac::{interrupt, CorePeripherals, Peripherals, ADC};
use hal::usb::UsbBus;
use hal::{gpio, prelude::*};

use usb_device::bus::UsbBusAllocator;

mod prandtladc;
use prandtladc::*;

static mut BUS_ALLOCATOR: Option<UsbBusAllocator<UsbBus>> = None;
static mut APPLICATION: Option<Application<'static, UsbBus, Delay, Led, Pwm0, PrandtlPumpFanAdc>> =
    None;

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

    // Setup the fan & pump pwm pins
    // TODO: Extract to function
    let _pump_ctrl_pwm0_pin = pins.pa04.into_mode::<hal::gpio::AlternateE>(); // pump ctrl pwm1
    let _fan_ctrl_pwm0_pin = pins.pa05.into_mode::<hal::gpio::AlternateE>(); // fan ctrl pwm01

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

    // Setup PWM for pump and fan
    // TODO: Extract to fn
    let gclk = clocks.gclk0();
    let tcc0_tcc1_clock: &hal::clock::Tcc0Tcc1Clock = &clocks.tcc0_tcc1(&gclk).unwrap();
    let mut pump_pwm = hal::pwm::Pwm0::new(
        &tcc0_tcc1_clock,
        1u32.kHz(),
        peripherals.TCC0,
        &mut peripherals.PM,
    );

    let mut adc = Adc::adc(peripherals.ADC, &mut peripherals.PM, &mut clocks);
    let mut pump_sense_channel = pins.pa06.into_mode::<gpio::AlternateB>();
    let mut fan_sense_channel = pins.pa07.into_mode::<gpio::AlternateB>();

    let padc = PrandtlPumpFanAdc::new(adc, pump_sense_channel, fan_sense_channel);

    // NOTE: This must happen before we enable USB interrupt.
    unsafe {
        APPLICATION = Some(Application::new(
            BUS_ALLOCATOR.as_ref().unwrap(),
            delay,
            led,
            pump_pwm,
            Channel::_0,
            Channel::_1,
            padc,
        ));
    }

    // this stays
    unsafe {
        core.NVIC.set_priority(interrupt::USB, 1);
        NVIC::unmask(interrupt::USB);
    }
}

// TODO: Finish feature parity with RTIC version.
// [x] PWM Pump/Fan control
// [x] ADC Pump/Fan input
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
                        let pump_pwm_duty_norm =
                            (control_packet.pump_control_voltage as f32) / 100.0;
                        let pump_pwm_duty =
                            (pump_pwm_duty_norm * (app.pump_pwm.get_max_duty() as f32)) as u32;

                        app.pump_pwm.set_duty(Channel::_0, pump_pwm_duty);
                        app.pump_pwm.set_duty(Channel::_1, pump_pwm_duty);
                        //app.led.set_state(control_packet.command.into());
                    }
                    _ => {}
                }
            }

            // NOTE: DEBUG CODE
            for i in 0..2 {
                let pump_speed = app.padc.read_pump_sense_raw();
                let fan_speed = app.padc.read_fan_sense_raw();

                if let Some(pump_speed) = pump_speed {
                    let pump_speed_norm = (pump_speed as f32) / (4096f32);
                    app.led.set_state((pump_speed_norm < 0.5f32).into());

                    if let Some(fan_speed) = fan_speed {
                        app.outgoing_packets.push(Packet::ReportSensors(
                            common::packet::ReportSensorsPacket {
                                pump_speed_norm: pump_speed,
                                fan_speed_norm: fan_speed,
                                valve_state: common::packet::ValveState::Open,
                            },
                        ));
                        continue;
                    }
                }

                app.outgoing_packets.push(Packet::ReportSensors(
                    common::packet::ReportSensorsPacket {
                        pump_speed_norm: 1,
                        fan_speed_norm: 1,
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
