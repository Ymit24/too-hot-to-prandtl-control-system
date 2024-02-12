use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Voltage<const MAX: u8> {
    pub value: f32,
}

#[derive(Error, Debug)]
pub enum VoltageError {
    #[error("Invalid Range")]
    InvalidRange,
}

impl<const MAX: u8> Voltage<MAX> {
    /// Convert to a normalized u8 (0 volts = 0 & MAX volts = 255)
    pub fn into_norm(self) -> u8 {
        ((self.value / (MAX as f32)) * (u8::max_value() as f32)) as u8
    }
}

impl<const MAX: u8> TryFrom<f32> for Voltage<MAX> {
    type Error = VoltageError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value > (MAX as f32) {
            return Err(VoltageError::InvalidRange);
        }

        Ok(Self { value })
    }
}

impl<const MAX: u8> Display for Voltage<MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} V)", self.value)
    }
}
