use crate::{internals::core::ports::HostSensorPort, models::host_sensor_data::HostSensorData};

use super::services::HostCpuTemperatureService;

pub struct HostSensorAdapter<'a> {
    pub service: &'a dyn HostCpuTemperatureService,
}

impl<'a> HostSensorAdapter<'a> {
    pub fn new(service: &'a dyn HostCpuTemperatureService) -> Self {
        Self { service }
    }
}

impl<'a> HostSensorPort for HostSensorAdapter<'a> {
    fn poll_host_sensors(&self) -> HostSensorData {
        let cpu_temperature = self.service.get_cpu_temp();
        HostSensorData { cpu_temperature }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        externals::host_sensors::services::HostCpuTemperatureService,
        internals::core::ports::HostSensorPort, models::temperature::Temperature,
    };

    use super::HostSensorAdapter;
    static MOCK_CPU_TEMP: Temperature = Temperature { value: 100f32 };

    struct MockService;
    impl HostCpuTemperatureService for MockService {
        fn get_cpu_temp(&self) -> Temperature {
            MOCK_CPU_TEMP
        }
    }

    #[test]
    fn test_host_sensor_adapter() {
        let mock_service = MockService {};
        let dut = HostSensorAdapter::new(&mock_service);
        let results = dut.poll_host_sensors();

        assert_eq!(results.cpu_temperature, MOCK_CPU_TEMP);
    }
}
