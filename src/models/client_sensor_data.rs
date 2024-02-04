use std::fmt::Display;

use super::rpm::Rpm;

#[derive(Debug, Clone, Copy)]
pub struct ClientSensorData {
    pub pump_speed: Rpm<3200>, // NOTE: placeholder
}

impl Display for ClientSensorData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(ClientSensorData: pump_speed={})", self.pump_speed)
    }
}
