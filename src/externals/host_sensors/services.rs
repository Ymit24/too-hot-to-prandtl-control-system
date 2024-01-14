use crate::models::temperature::Temperature;

pub trait HostCpuTemperatureService {
    fn get_cpu_temp(&self) -> Temperature;
}

pub struct HostCpuTemperatureServiceActual;

impl HostCpuTemperatureService for HostCpuTemperatureServiceActual {
    fn get_cpu_temp(&self) -> Temperature {
        unimplemented!()
    }
}
