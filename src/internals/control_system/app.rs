use thiserror::Error;

use super::{models::Temperature, ports::HostSensorPort};

pub struct Application<T: HostSensorPort> {
    host_sensor_port: T,
}

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Unexpected Error")]
    Unexpected,
}

impl<T: HostSensorPort> Application<T> {
    pub fn new(host_sensor_port: T) -> Self {
        return Self { host_sensor_port };
    }

    pub fn run(&self) -> Result<(), ApplicationError> {
        let sensors = self.host_sensor_port.get_host_sensor_data();

        if sensors.cpu_temperature.value > 10f32 {}

        Ok(())
    }
}
