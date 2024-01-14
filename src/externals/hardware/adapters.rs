use crate::{
    internals::core::ports::{ClientSensorPort, ControlEventPort},
    models::{client_sensor_data::ClientSensorData, control_event::ControlEvent},
};

pub struct PollClientSensorAdapter;
pub struct EmitToHardwareAdapter;

impl ClientSensorPort for PollClientSensorAdapter {
    fn poll_client_sensors(&self) -> ClientSensorData {
        unimplemented!()
    }
}

impl ControlEventPort for EmitToHardwareAdapter {
    fn emit(&self, event: ControlEvent) {
        unimplemented!()
    }
}
