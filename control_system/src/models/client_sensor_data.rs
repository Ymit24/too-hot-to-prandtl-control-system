use std::fmt::Display;

use common::{packet::ReportSensorsPacket, physical::Rpm};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct ClientSensorData {
    pub pump_speed: Rpm,
}

#[derive(Error, Debug)]
pub enum ClientSensorDataError {
    #[error("Generic catch all error.")]
    Invalid,
}

impl Display for ClientSensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(ClientSensorData: pump_speed={})", self.pump_speed)
    }
}

impl TryFrom<ReportSensorsPacket> for ClientSensorData {
    type Error = ClientSensorDataError;

    fn try_from(value: ReportSensorsPacket) -> Result<Self, Self::Error> {
        Ok(ClientSensorData {
            pump_speed: value.pump_speed_norm,
        })
    }
}
