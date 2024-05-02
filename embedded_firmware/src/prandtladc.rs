use crate::hal::prelude::*;
use atsamd_hal::{
    adc::Adc,
    gpio::{Alternate, Pin, B, PA06, PA07},
    pac::ADC,
};
use embedded_firmware_core::{convert_raw_to_normalized, PrandtlAdc};

pub type PumpPin = Pin<PA06, Alternate<B>>;
pub type FanPin = Pin<PA07, Alternate<B>>;

pub struct PrandtlPumpFanAdc {
    adc: Adc<ADC>,
    pump_sense_channel: PumpPin,
    fan_sense_channel: FanPin,
    resolution: u8,
}

impl PrandtlPumpFanAdc {
    pub fn new(
        adc: Adc<ADC>,
        pump_sense_channel: PumpPin,
        fan_sense_channel: FanPin,
        resolution: u8,
    ) -> Self {
        Self {
            adc,
            pump_sense_channel,
            fan_sense_channel,
            resolution,
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

    fn read_pump_sense_norm(&mut self) -> Option<f32> {
        self.read_pump_sense_raw()
            .map(|raw| convert_raw_to_normalized(raw, self.resolution))
    }

    fn read_fan_sense_norm(&mut self) -> Option<f32> {
        self.read_fan_sense_raw()
            .map(|raw| convert_raw_to_normalized(raw, self.resolution))
    }
}
