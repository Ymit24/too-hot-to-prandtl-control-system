use super::temperature::Temperature;

#[derive(Debug,Clone,Copy)]
pub struct HostSensorData {
    pub cpu_temperature: Temperature,
}
