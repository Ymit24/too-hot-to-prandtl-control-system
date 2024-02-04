use std::fmt::Display;

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Temperature {
    pub value: f32,
}

#[derive(Error, Debug)]
pub enum TemperatureError {
    #[error("Temperature too high")]
    TooHigh,
}

impl TryFrom<f32> for Temperature {
    type Error = TemperatureError;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value > 100f32 {
            return Err(TemperatureError::TooHigh);
        }
        Ok(Temperature { value })
    }
}

impl Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} degC)", self.value)
    }
}
