use common::{
    packet::{Packet, ReportControlTargetsPacket},
    physical::{Percentage, ValveState},
};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct ControlEvent {
    pub fan_activation: Percentage,  // NOTE: placeholder
    pub pump_activation: Percentage, // NOTE: placeholder
    pub valve_state: ValveState,
}

#[derive(Error, Debug)]
pub enum ControlEventError {
    #[error("Invalid Range")]
    InvalidRange,
}

impl Display for ControlEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<Control Event | fan_speed:{}, pump_pwm:{}, valve_state:{}>",
            self.fan_activation, self.pump_activation, self.valve_state
        )
    }
}

impl TryFrom<ControlEvent> for Packet {
    type Error = ControlEventError;

    fn try_from(value: ControlEvent) -> Result<Self, Self::Error> {
        Ok(Packet::ReportControlTargets(ReportControlTargetsPacket {
            fan_control_percent: value.fan_activation,
            pump_control_percent: value.pump_activation,
            valve_control_state: value.valve_state,
        }))
    }
}
