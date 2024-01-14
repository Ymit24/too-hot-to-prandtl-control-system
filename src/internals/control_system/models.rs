use thiserror::Error;

pub struct HostSensorData {
    pub cpu_temperature: Temperature,
    pub cpu_frequencies: Vec<ClockFrequency>,
}

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

pub struct ClockFrequency {
    pub megahertz: u16,
}

#[derive(Error, Debug)]
pub enum ClockFrequencyError {
    #[error("Clock Frequency Out Of Valid Range")]
    OutOfRange,
}

impl TryFrom<u16> for ClockFrequency {
    type Error = ClockFrequencyError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 5000 {
            return Err(ClockFrequencyError::OutOfRange);
        }
        Ok(ClockFrequency { megahertz: value })
    }
}
