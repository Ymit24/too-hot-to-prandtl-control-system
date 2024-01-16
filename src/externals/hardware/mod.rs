use self::{
    adapters::{EmitToHardwareAdapter, PollClientSensorAdapter},
    services::{HardwareService, HardwareServiceUsb, HeartbeatMessage},
};

pub mod adapters;
pub mod services;

pub struct HardwareModule {
    pub client_sensor_adapter: PollClientSensorAdapter<'static>,
    pub control_event_adapter: EmitToHardwareAdapter,
}

static HARDWARE_SERVICE_USB: HardwareServiceUsb = HardwareServiceUsb::new();

impl HardwareModule {
    pub fn initialize() -> Self {
        let mut service = HardwareServiceUsb::new().unwrap();
        service.queue_message(&HeartbeatMessage::new());

        Self {
            client_sensor_adapter: PollClientSensorAdapter::new(&service),
            control_event_adapter: EmitToHardwareAdapter {},
        }
    }
}
