use crate::internals::control_system::{models::HostSensorData, ports::HostSensorPort};

pub struct HostSensorAdapter {}

impl HostSensorPort for HostSensorAdapter {
    fn get_host_sensor_data(&self) -> HostSensorData {
        todo!()
    }
}
