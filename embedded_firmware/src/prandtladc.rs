use crate::hal::prelude::*;
use atsamd_hal::{
    adc::Adc,
    gpio::{Alternate, Pin, B, PA04, PA06},
    pac::ADC,
};
use embedded_firmware_core::PrandtlAdc;

pub type PumpPin = Pin<PA06, Alternate<B>>;
pub type FanPin = Pin<PA04, Alternate<B>>;

pub struct PrandtlPumpFanAdc {
    adc: Adc<ADC>,
    pump_sense_channel: PumpPin,
    fan_sense_channel: FanPin,
}

impl PrandtlPumpFanAdc {
    pub fn new(adc: Adc<ADC>, pump_sense_channel: PumpPin, fan_sense_channel: FanPin) -> Self {
        Self {
            adc,
            pump_sense_channel,
            fan_sense_channel,
        }
    }
}

impl PrandtlAdc for PrandtlPumpFanAdc {
    fn read_pump_sense_raw(&mut self) -> Option<u16> {
        if let Ok(value) = self.adc.read(&mut self.pump_sense_channel) {
            return Some(value);
        }
        None
    }

    fn read_fan_sense_raw(&mut self) -> Option<u16> {
        if let Ok(value) = self.adc.read(&mut self.fan_sense_channel) {
            return Some(value);
        }
        None
    }
}
