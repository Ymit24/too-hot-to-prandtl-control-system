#![cfg_attr(not(test), no_std)]
use common::physical::RpmError;
use thiserror_no_std::Error;

pub trait PrandtlAdc {
    fn read_pump_sense_raw(&mut self) -> Option<u16>;
    fn read_fan_sense_raw(&mut self) -> Option<u16>;

    fn read_pump_sense_norm(&mut self) -> Option<f32>;
    fn read_fan_sense_norm(&mut self) -> Option<f32>;
}

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Failed to pump or fan speed from adc.")]
    ReadAdcFailure,
    #[error("Failed to read valve state from pins.")]
    ValveReadFailure,
    #[error("Rpm related error.")]
    RpmError(RpmError),
}

/// Convert a 0 -> 2^resolution into a 0 to 1 value.
pub fn convert_raw_to_normalized(raw: u16, resolution: u8) -> f32 {
    (raw as f32) / (2i32.pow(resolution as u32) as f32)
}

pub mod application;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_raw_to_normalized() {
        assert_eq!(0f32, convert_raw_to_normalized(0, 12));
        assert_eq!(0.5f32, convert_raw_to_normalized(4096 / 2, 12));
        assert_eq!(1f32, convert_raw_to_normalized(4096, 12));
    }
}
