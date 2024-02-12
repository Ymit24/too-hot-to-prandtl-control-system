use super::{rpm::Rpm, voltage::Voltage};
use common::packet::{Packet, ReportControlTargetsPacket, ValveState};
use std::fmt::Display;
use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub struct ControlEvent {
    pub fan_speed: Rpm<2300>, // NOTE: placeholder
    pub pump_pwm: Voltage<5>, // NOTE: placeholder
    pub valve_state: ValveState,
    pub debug_command: bool, // NOTE: THIS IS A DEBUG COMMAND
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
            "<Control Event | fan_speed:{}, pump_pwm:{}>",
            self.fan_speed, self.pump_pwm
        )
    }
}

impl TryFrom<ControlEvent> for Packet {
    type Error = ControlEventError;

    fn try_from(value: ControlEvent) -> Result<Self, Self::Error> {
        Ok(Packet::ReportControlTargets(ReportControlTargetsPacket {
            fan_control_voltage: value.fan_speed.into_norm(),
            pump_control_voltage: value.pump_pwm.into_norm(),
            valve_control_state: value.valve_state.into(),
            command: value.debug_command,
        }))
    }
}
