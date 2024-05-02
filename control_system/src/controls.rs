use common::{packet::ValveState, physical::Percentage};

use crate::models::{
    client_sensor_data::ClientSensorData, control_event::ControlEvent,
    host_sensor_data::HostSensorData,
};

pub fn generate_control_frame(
    client_sensor_data: ClientSensorData,
    host_sensor_data: HostSensorData,
) -> ControlEvent {
    // TODO: REMOVE THIS DEBUG CODE
    ControlEvent {
        fan_activation: 100f32.try_into().expect("Failed to get percentage."),
        pump_activation: 50f32.try_into().expect("Failed to get percentage."),
        valve_state: ValveState::Open,
    }
}

#[cfg(test)]
mod testing {
    use crate::models::{rpm::Rpm, temperature::Temperature};

    use super::*;
}
