use crate::models::{
    client_sensor_data::ClientSensorData, control_event::ControlEvent,
    host_sensor_data::HostSensorData,
};

pub fn generate_control_frame(
    client_sensor_data: ClientSensorData,
    host_sensor_data: HostSensorData,
) -> ControlEvent {
    unimplemented!()
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
