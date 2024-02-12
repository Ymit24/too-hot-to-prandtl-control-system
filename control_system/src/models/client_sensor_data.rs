use std::fmt::Display;

use super::rpm::{Rpm, RpmError};
use common::packet::ReportSensorsPacket;
use thiserror::Error;

const PUMP_RPM: u16 = 3200;
type PumpRpm = Rpm<PUMP_RPM>;

#[derive(Debug, Clone, Copy)]
pub struct ClientSensorData {
    pub pump_speed: Rpm<PUMP_RPM>, // NOTE: placeholder
}

#[derive(Error, Debug)]
pub enum ClientSensorDataError {
    #[error("Generic catch all error.")]
    Invalid,
    #[error("RpmError")]
    RpmError(RpmError),
}

impl From<RpmError> for ClientSensorDataError {
    fn from(value: RpmError) -> Self {
        Self::RpmError(value)
    }
}

impl Display for ClientSensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(ClientSensorData: pump_speed={})", self.pump_speed)
    }
}

impl TryFrom<ReportSensorsPacket> for ClientSensorData {
    type Error = ClientSensorDataError;

    fn try_from(value: ReportSensorsPacket) -> Result<Self, Self::Error> {
        let pump_rpm: PumpRpm = match Rpm::try_from_norm(value.pump_speed_norm) {
            Err(e) => return Err(e.into()),
            Ok(rpm) => rpm,
        };
        // 0>= pump_speed_rpm >= 3200
        Ok(ClientSensorData {
            pump_speed: pump_rpm,
        })
    }
}
