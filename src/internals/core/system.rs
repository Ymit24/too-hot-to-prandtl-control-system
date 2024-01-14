use super::{
    controls::generate_control_frame,
    ports::{ClientSensorPort, ControlEventPort, HostSensorPort, TuningPort},
};

pub type MultiPort<'a> = Vec<&'a dyn ControlEventPort>;

pub struct CoreSystem<'a, A: ClientSensorPort, B: HostSensorPort, C: TuningPort> {
    pub client_sensor_port: A,
    pub host_sensor_port: B,
    pub tuning_port: C,
    // pub control_event_ports: Vec<&'a dyn ControlEventPort>,
    pub control_event_ports: MultiPort<'a>,
}

impl<'a, A: ClientSensorPort, B: HostSensorPort, C: TuningPort> CoreSystem<'a, A, B, C> {
    pub fn new(
        client_sensor_port: A,
        host_sensor_port: B,
        tuning_port: C,
        control_event_port: Vec<&'a dyn ControlEventPort>,
    ) -> Self {
        CoreSystem {
            client_sensor_port,
            host_sensor_port,
            tuning_port,
            control_event_ports: control_event_port,
        }
    }

    pub fn tick(&self) {
        let client_sensor_data = self.client_sensor_port.poll_client_sensors();
        let host_sensor_data = self.host_sensor_port.poll_host_sensors();

        // TODO: add tuning information

        let control_event = generate_control_frame(client_sensor_data, host_sensor_data);

        for port in self.control_event_ports.iter() {
            port.emit(control_event);
        }
    }
}
