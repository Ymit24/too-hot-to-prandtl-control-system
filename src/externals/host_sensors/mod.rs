use self::adapters::HostSensorAdapter;

pub mod adapters;

pub struct HostSensorModule {
    pub host_sensor_adapter: HostSensorAdapter,
}

impl HostSensorModule {
    pub fn initialize() -> Self {
        Self {
            host_sensor_adapter: HostSensorAdapter {},
        }
    }
}
