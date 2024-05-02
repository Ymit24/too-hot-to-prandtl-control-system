use std::fmt::Display;

use common::{
    packet::ReportSensorsPacket,
    physical::{Rpm, ValveState},
};
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct ClientSensorData {
    pub pump_speed: Rpm,
    pub fan_speed: Rpm,
    pub valve_state: ValveState,
}

#[derive(Error, Debug)]
pub enum ClientSensorDataError {
    #[error("Generic catch all error.")]
    Invalid,
}

impl Display for ClientSensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "(ClientSensorData: pump_speed={}, fan_speed={}, valve_state={})",
            self.pump_speed, self.fan_speed, self.valve_state
        )
    }
}

impl TryFrom<ReportSensorsPacket> for ClientSensorData {
    type Error = ClientSensorDataError;

    fn try_from(value: ReportSensorsPacket) -> Result<Self, Self::Error> {
        Ok(ClientSensorData {
            pump_speed: value.pump_speed_rpm,
            fan_speed: value.fan_speed_rpm,
            valve_state: value.valve_state,
        })
    }
}
