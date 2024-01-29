use super::rpm::Rpm;

#[derive(Debug,Clone,Copy)]
pub struct ClientSensorData {
    pub pump_speed: Rpm<3200>, // NOTE: placeholder
}
