use std::fmt::Display;

use super::{rpm::Rpm, voltage::Voltage};

#[derive(Debug,Clone, Copy)]
pub struct ControlEvent {
    pub fan_speed: Rpm<2300>, // NOTE: placeholder
    pub pump_pwm: Voltage<5>, // NOTE: placeholder
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
