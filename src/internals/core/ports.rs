use crate::models::{
    client_sensor_data::ClientSensorData, control_event::ControlEvent,
    host_sensor_data::HostSensorData,
};

pub trait ClientSensorPort {
    fn poll_client_sensors(&self) -> ClientSensorData;
}

pub trait HostSensorPort {
    fn poll_host_sensors(&self) -> HostSensorData;
}

pub trait ControlEventPort {
    fn emit(&self, event: ControlEvent);
}

pub trait TuningPort {
    fn poll_tuning(&self);
}
