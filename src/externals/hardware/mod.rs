use self::adapters::{EmitToHardwareAdapter, PollClientSensorAdapter};

pub mod adapters;

pub struct HardwareModule {
    pub client_sensor_adapter: PollClientSensorAdapter,
    pub control_event_adapter: EmitToHardwareAdapter,
}

impl HardwareModule {
    pub fn initialize() -> Self {
        Self {
            client_sensor_adapter: PollClientSensorAdapter {},
            control_event_adapter: EmitToHardwareAdapter {},
        }
    }
}
