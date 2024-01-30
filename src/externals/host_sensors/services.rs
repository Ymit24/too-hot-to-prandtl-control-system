use std::io;

use crate::models::temperature::{Temperature, TemperatureError};
use anyhow::Result;
use systemstat::{Platform, System};
use thiserror::Error;

pub trait HostCpuTemperatureService {
    fn get_cpu_temp(&self) -> Result<Temperature, CpuTemperatureServiceError>;
}

pub struct HostCpuTemperatureServiceActual;

#[derive(Error, Debug)]
pub enum CpuTemperatureServiceError {
    #[error("Failed to read cpu temperature.")]
    FailedToRead(io::Error),
    #[error("Failed to parse cpu temperature.")]
    FailedToParse(TemperatureError),
}

impl HostCpuTemperatureService for HostCpuTemperatureServiceActual {
    fn get_cpu_temp(&self) -> Result<Temperature, CpuTemperatureServiceError> {
        let raw = match System::new().cpu_temp() {
            Ok(t) => t,
            Err(e) => return Err(CpuTemperatureServiceError::FailedToRead(e)),
        };

        Temperature::try_from(raw).map_err(|e| CpuTemperatureServiceError::FailedToParse(e))
    }
}
