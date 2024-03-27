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

    // NOTE: This is a 3v3 ADC. 0V -> 0 3.3V -> 4096
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

        app.core_loop();

        app.delay.delay_ms(100u16);
    }
}

#[interrupt]
fn USB() {
    unsafe {
        APPLICATION.as_mut().unwrap().poll_usb();
    }
}
