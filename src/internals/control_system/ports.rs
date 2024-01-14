use crate::internals::control_system::models::HostSensorData;

pub trait HostSensorPort {
    fn get_host_sensor_data(&self) -> HostSensorData;
}
