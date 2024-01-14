use crate::{internals::core::ports::HostSensorPort, models::host_sensor_data::HostSensorData};

pub struct HostSensorAdapter;

impl HostSensorPort for HostSensorAdapter {
    fn poll_host_sensors(&self) -> HostSensorData {
        unimplemented!()
    }
}
