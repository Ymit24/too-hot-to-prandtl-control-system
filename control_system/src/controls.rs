use common::physical::{Percentage, Rpm, ValveState};
use once_cell::sync::Lazy;

use crate::models::{
    client_sensor_data::ClientSensorData, control_event::ControlEvent, curve::Curve,
    host_sensor_data::HostSensorData, temperature::Temperature,
};

const PUMP_CURVE: Lazy<Curve<Temperature, Percentage>> = Lazy::new(|| {
    Curve::new(vec![
        (
            0f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(30f32).expect("Failed to get percentage."),
        ),
        (
            50f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(30f32).expect("Failed to get percentage."),
        ),
        (
            80f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(90f32).expect("Failed to get percentage."),
        ),
        (
            85f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(100f32).expect("Failed to get percentage."),
        ),
    ])
    .expect("Failed to get pump curve.")
});

const FAN_CURVE: Lazy<Curve<Temperature, Percentage>> = Lazy::new(|| {
    Curve::new(vec![
        (
            0f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(15f32).expect("Failed to get percentage."),
        ),
        (
            60f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(15f32).expect("Failed to get percentage."),
        ),
        (
            85f32.try_into().expect("Failed to get temperature."),
            Percentage::try_from(100f32).expect("Failed to get percentage."),
        ),
    ])
    .expect("Failed to get fan curve.")
});

const VALVE_CURVE: Lazy<Curve<Temperature, ValveState>> = Lazy::new(|| {
    Curve::new(vec![
        (
            0f32.try_into().expect("Failed to get temperature."),
            ValveState::Open,
        ),
        (
            59f32.try_into().expect("Failed to get temperature."),
            ValveState::Open,
        ),
        (
            60f32.try_into().expect("Failed to get temperature."),
            ValveState::Closed,
        ),
    ])
    .expect("Failed to get valve curve.")
});

/// Closed loop feedback sensitivity K.
/// Higher value means more sensitive;
const PUMP_SENSITIVITY_K: f32 = 0.15f32;

pub fn generate_control_frame(
    client_sensor_data: ClientSensorData,
    host_sensor_data: HostSensorData,
) -> ControlEvent {
    let temperature = host_sensor_data.cpu_temperature;
    let target_pump_percent = pump_controller(temperature, client_sensor_data.pump_speed);
    let target_fan_percent = match FAN_CURVE.lookup(temperature) {
        None => {
            tracing::error!(
                "Failed to get fan value for temperature {}. Defaulting to 100%!",
                temperature
            );
            Percentage::try_from(100f32).expect("Failed to get percentage.")
        }
        Some(percentage) => percentage,
    };
    let target_valve_state = match VALVE_CURVE.lookup(temperature) {
        None => {
            tracing::error!(
                "Failed to get valve value for temperature {}. Defaulting to Open!",
                temperature
            );
            ValveState::Open
        }
        Some(percentage) => percentage,
    };

    ControlEvent {
        fan_activation: target_fan_percent,
        pump_activation: target_pump_percent,
        valve_state: target_valve_state,
    }
}

/// Apply the `Pump Controller` control system.
fn pump_controller(temperature: Temperature, pump_rpm: Rpm) -> Percentage {
    let target_activation = match PUMP_CURVE.lookup(temperature) {
        None => {
            tracing::error!(
                "Failed to get pump value for temperature {}. Defaulting to 100%!",
                temperature
            );
            Percentage::try_from(100f32).expect("Failed to get percentage.")
        }
        Some(percentage) => percentage,
    };
    let raw_current_speed_percentage: f32 = pump_rpm.into_percentage().into();
    let raw_target: f32 = target_activation.into();

    let raw_feedback_target = apply_feedback(raw_current_speed_percentage, raw_target);
    match Percentage::try_from(raw_feedback_target) {
        Err(err) => {
            tracing::warn!("Failed to convert target activation percentage into `Percentage`. Clamping to min/max bounds. Error: {}", err);
            Percentage::try_from(raw_current_speed_percentage.clamp(0f32, 100f32))
                .expect("Failed to get Percentage.")
        }
        Ok(perc) => perc,
    }
}

/// Apply basic feedback with `PUMP_SENSITIVITY_K` parameter.
fn apply_feedback(current: f32, target: f32) -> f32 {
    current + ((target - current) * PUMP_SENSITIVITY_K)
}

#[cfg(test)]
mod testing {
    use common::physical::Rpm;

    use super::*;

    #[test]
    fn test_generate_control_frame() {
        let client = ClientSensorData {
            pump_speed: Rpm::new(500f32, 500f32).expect("Failed to get RPM."),
            fan_speed: Rpm::new(500f32, 500f32).expect("Failed to get RPM."),
            valve_state: ValveState::Open,
        };

        for i in 0..100 {
            let host = HostSensorData {
                cpu_temperature: Temperature::try_from(i as f32)
                    .expect("Failed to get Temperature."),
            };

            let control_frame = generate_control_frame(client, host);

            assert_eq!(
                control_frame.fan_activation,
                FAN_CURVE
                    .lookup(host.cpu_temperature)
                    .expect("Failed to get curve value.")
            );
            let raw_current_pump_speed = client.pump_speed.into_percentage().into();
            let raw_target = PUMP_CURVE
                .lookup(host.cpu_temperature)
                .expect("Failed to get curve value.")
                .into();
            assert_eq!(
                control_frame.pump_activation,
                Percentage::try_from(apply_feedback(raw_current_pump_speed, raw_target))
                    .expect("Failed to get Percentage.")
            );
            assert_eq!(
                control_frame.valve_state,
                VALVE_CURVE
                    .lookup(host.cpu_temperature)
                    .expect("Failed to get curve value.")
            );
        }
    }

    #[test]
    fn test_apply_feedback() {
        for current in 0..100 {
            for target in 0..100 {
                let current = current as f32;
                let target = target as f32;
                let result = apply_feedback(current, target);

                let correct = current + ((target - current) * PUMP_SENSITIVITY_K);

                assert_eq!(result, correct);
            }
        }
    }
}
