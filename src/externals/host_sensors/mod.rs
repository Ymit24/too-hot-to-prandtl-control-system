use self::{adapters::HostSensorAdapter, services::HostCpuTemperatureServiceActual};

pub mod adapters;
pub mod services;

pub struct HostSensorModule {
    pub host_sensor_adapter: HostSensorAdapter<'static>,
}

static HOST_TEMPERATURE_SERVICE: HostCpuTemperatureServiceActual =
    HostCpuTemperatureServiceActual {};

impl HostSensorModule {
    pub fn initialize() -> Self {
        Self {
            host_sensor_adapter: HostSensorAdapter {
                service: &HOST_TEMPERATURE_SERVICE,
            },
        }
    }
}
