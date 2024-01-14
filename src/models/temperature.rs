use thiserror::Error;

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
