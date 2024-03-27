use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct Rpm<const MAX: u16> {
    pub value: u16,
}

#[derive(Error, Debug)]
pub enum RpmError {
    #[error("Invalid Range")]
    InvalidRange,
}

impl<const MAX: u16> TryFrom<u16> for Rpm<MAX> {
    type Error = RpmError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > MAX {
            return Err(RpmError::InvalidRange);
        }
        Ok(Self { value })
    }
}

impl<const MAX: u16> Display for Rpm<MAX> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} rpm)", self.value)
    }
}

impl<const MAX: u16> Rpm<MAX> {
    pub fn try_from_norm(norm: u16) -> Result<Self, RpmError> {
        Self::try_from((norm / 255u16) as u16)
    }

    /// Convert to a normalized u8 (0 rpm = 0 & MAX rpm = 255)
    pub fn into_norm(self) -> u8 {
        (((self.value as f32) / (u16::max_value() as f32)) * (u8::max_value() as f32)) as u8
    }
}
