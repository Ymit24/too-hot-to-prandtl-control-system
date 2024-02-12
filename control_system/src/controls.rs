use std::{
    cell::{Cell, RefCell},
    sync::Arc,
    time::Instant,
};

use common::packet::ValveState;
use once_cell::sync::OnceCell;

use crate::models::{
    client_sensor_data::ClientSensorData, control_event::ControlEvent,
    host_sensor_data::HostSensorData,
};

pub fn generate_control_frame(
    client_sensor_data: ClientSensorData,
    host_sensor_data: HostSensorData,
) -> ControlEvent {
    // TODO: REMOVE THIS DEBUG CODE
    let state = rand::random();
    tracing::info!("Current led state: {}", state);
    let v1: bool = rand::random();
    let v2: bool = rand::random();
    let v = v1 as u8 + v2 as u8;
    ControlEvent {
        fan_speed: crate::models::rpm::Rpm { value: 1250 },
        pump_pwm: crate::models::voltage::Voltage {
            value: v as f32 * 2.5f32,
        },
        valve_state: ValveState::Open,
        debug_command: state,
    }
}

#[cfg(test)]
mod testing {
    use crate::models::{rpm::Rpm, temperature::Temperature, voltage::Voltage};

    use super::*;

    #[test]
    fn test_generate_control_frame() {
        // NOTE: EXAMPLE TEST

        todo!("Write actual test!");

        let client = ClientSensorData {
            pump_speed: Rpm::try_from(3100).expect("Failed to generate rpm"),
        };
        let host = HostSensorData {
            cpu_temperature: Temperature::try_from(70f32).expect("Failed to generate temperature"),
        };

        let _results = generate_control_frame(client, host);
    }
}
