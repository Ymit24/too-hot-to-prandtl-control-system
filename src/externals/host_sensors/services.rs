use std::io;

use crate::models::temperature::{Temperature, TemperatureError};
use anyhow::Result;
use systemstat::{Platform, System};
use thiserror::Error;

/// This service allows separation of the external logic of getting
/// the cpu temperature from the business logic which makes the system
/// easier to unit test.
pub trait HostCpuTemperatureService {
    /// Attempt to get the current cpu temperature and convert it into
    /// a Temperature model. Will return an appropriate error if it is not
    /// able to.
    fn get_cpu_temp(&self) -> Result<Temperature, CpuTemperatureServiceError>;
}

pub struct HostCpuTemperatureServiceActual;

#[derive(Error, Debug)]
pub enum CpuTemperatureServiceError {
    /// This occurs if systemstat fails to report the temperature.
    #[error("Failed to read cpu temperature.")]
    FailedToRead(io::Error),

    /// This occurs if the Temperature model fails to parse the raw f32 temperature.
    #[error("Failed to parse cpu temperature.")]
    FailedToParse(TemperatureError),
}

impl HostCpuTemperatureService for HostCpuTemperatureServiceActual {
    /// Use systemstat crate to provide platform specific implementations
    /// of get_cpu. Will convert raw f32 temperature into a Temperature model.
    /// Will return a FailedToRead error with the io::Error if systemstat fails
    /// to get the raw cpu temperature. Will return a FailedToParse with the
    /// TemperatureError if the raw cpu temperature fails to parse into a
    /// Temperature model.
    fn get_cpu_temp(&self) -> Result<Temperature, CpuTemperatureServiceError> {
        let raw = match System::new().cpu_temp() {
            Ok(t) => t,
            Err(e) => return Err(CpuTemperatureServiceError::FailedToRead(e)),
        };

        Temperature::try_from(raw).map_err(|e| CpuTemperatureServiceError::FailedToParse(e))
    }
}
