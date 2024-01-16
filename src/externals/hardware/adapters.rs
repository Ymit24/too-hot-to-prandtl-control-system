use crate::{
    internals::core::ports::{ClientSensorPort, ControlEventPort},
    models::{client_sensor_data::ClientSensorData, control_event::ControlEvent},
};

use super::services::HardwareService;

pub struct PollClientSensorAdapter<'a> {
    pub service: &'a dyn HardwareService,
}

pub struct EmitToHardwareAdapter;

impl<'a> PollClientSensorAdapter<'a> {
    pub fn new(service: &'a dyn HardwareService) -> Self {
        Self { service }
    }
}

impl<'a> ClientSensorPort for PollClientSensorAdapter<'a> {
    fn poll_client_sensors(&self) -> ClientSensorData {
        unimplemented!()
    }
}

impl ControlEventPort for EmitToHardwareAdapter {
    fn emit(&self, event: ControlEvent) {
        unimplemented!()
    }
}
