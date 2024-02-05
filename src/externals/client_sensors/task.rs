use futures::{Future, StreamExt};
use serialport::{SerialPort, SerialPortInfo};
use std::time::Duration;
use tokio::{select, sync::broadcast::Sender};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::models::client_sensor_data::ClientSensorData;

pub enum State {
    Passive,
    Active,
}

/// Client Sensor FSM
pub struct ClientHardwareFSM {
    state: State,
    tasks: TaskTracker,
    state_token: CancellationToken,
}

impl ClientHardwareFSM {
    pub fn new() -> Self {
        Self {
            state: State::Passive,
            tasks: TaskTracker::new(),
            state_token: CancellationToken::new(),
        }
    }

    #[instrument(skip_all)]
    pub async fn task_client_sensor_fsm(
        &mut self,
        token: CancellationToken,
        tx_client_sensor_data: Sender<ClientSensorData>,
    ) {
        info!("Started FSM.");
        loop {
            select! {
                _ = token.cancelled() => {
                    warn!("Canceled.");
                    break;
                }
            }
        }
    }

    /// Spawn the tasks required to operate this module.
    /// Internally handles states, cancellations, and cleanup.  
    pub fn spawn(
        &mut self,
        token: CancellationToken,
        tx_client_sensor_data: Sender<ClientSensorData>,
    ) {
        self.tasks.spawn(async {
            self.task_client_sensor_fsm(token, tx_client_sensor_data)
                .await
        });
    }

    #[tracing::instrument(skip_all)]
    async fn switch_state(&mut self, new_state: State) {
        // 1. Shutdown existing tasks by cancelling their state token
        self.state_token.cancel();
        self.tasks.close();
        self.tasks.wait().await;

        // 1b. Create new cancel token
        self.state_token = CancellationToken::new();

        // 2. Start up new tasks
        match new_state {
            State::Passive => {
                // 3a. Spawn tasks for passive mode
            }
            State::Active => {
                // 3b. Spawn tasks for active mode
            }
        }
        // 4. Update state
        self.state = new_state;
    }

    #[tracing::instrument(skip_all)]
    pub async fn task_find_client_port(&mut self, tx_state_change: Sender<State>) {
        loop {
            if self.state_token.is_cancelled() {
                info!("Task cancelled.");
                break; // NOTE: This ends the task
            }
            let ports = match serialport::available_ports() {
                Err(e) => {
                    error!("Failed to get any ports! Error: {}", e);
                    continue;
                }
                Ok(ports) => ports,
            };

            let processed_ports: Vec<(SerialPortInfo, bool)> = tokio_stream::iter(ports)
                .then(|port| async {
                    try_request_connection_for_port(self.state_token.clone(), port)
                })
                .buffered(5)
                .collect()
                .await;
            if processed_ports.is_empty() {
                warn!("Didn't find any candidate ports!");
                continue;
            }
            let candidate_ports: Vec<SerialPortInfo> = processed_ports
                .into_iter()
                .filter(|x| x.1 == true)
                .map(|x| x.0)
                .collect();

            let first_candidate_port = match candidate_ports.first() {
                None => {
                    warn!("No available ports which responded to connection attempt successfully!");
                    continue;
                }
                Some(candidate_port) => candidate_port,
            };

            info!(
                "Found candidate port which successfully responded to connection attempt. Name: {}",
                first_candidate_port.port_name
            );
            match tx_state_change.send(State::Active) {
                Err(e) => {
                    error!("Failed to send state change request. Error: {}", e);
                    continue;
                }
                Ok(_) => {
                    trace!("Successfully sent state change request.");
                    break; // NOTE: This ends the task.
                }
            }
        }
    }
}

/// Try and open communication with a port, send a request communication packet,
/// and receive an accept communication packet response. Returns true if all of these steps
/// pass and false if any of them fail.
async fn try_request_connection_for_port(
    token: CancellationToken,
    port: SerialPortInfo,
) -> (SerialPortInfo, bool) {
    (port, false)
}

// NOTE: MAYBE DON'T RETURN A STRING
async fn find_client_port(token: CancellationToken) -> Option<String> {
    let ports = match serialport::available_ports() {
        Err(e) => {
            error!("Failed to get any ports! Error: {}", e);
            return None;
        }
        Ok(ports) => ports,
    };
    let processed_ports: Vec<(SerialPortInfo, bool)> = tokio_stream::iter(ports)
        .then(|port| async { try_request_connection_for_port(token.clone(), port) })
        .buffered(5)
        .collect()
        .await;
    if processed_ports.is_empty() {
        warn!("Didn't find any candidate ports!");
        return None;
    }
    let candidate_ports: Vec<SerialPortInfo> = processed_ports
        .into_iter()
        .filter(|x| x.1 == true)
        .map(|x| x.0)
        .collect();

    let first_candidate_port = match candidate_ports.first() {
        None => {
            warn!("No available ports which responded to connection attempt successfully!");
            return None;
        }
        Some(candidate_port) => candidate_port,
    };

    info!(
        "Found candidate port which successfully responded to connection attempt. Name: {}",
        first_candidate_port.port_name
    );

    Some(first_candidate_port.port_name.clone())
}

async fn wait_for_client_port(token: CancellationToken) -> Result<String, String> {
    loop {
        if token.is_cancelled() {
            return Err(String::from("Canceled"));
        }
        match find_client_port(token.clone()).await {
            Some(port_name) => return Ok(port_name),
            None => continue,
        };
    }
}

// NOTE: STILL NEED TO HANDLE THE 'NEVER GIVE UP' CONCEPT.
#[tracing::instrument(skip_all)]
pub async fn task_poll_client_sensors(
    token: CancellationToken,
    tx_client_sensor_data: Sender<ClientSensorData>,
) {
    info!("Started.");

    let port_name = match wait_for_client_port(token.clone()).await {
        Err(e) => {
            warn!("Failed to wait for a client port. Cancelling. Error: {}", e);
            token.cancel();
            return;
        }
        Ok(port_name) => port_name,
    };

    // NOTE: MIGHT NOT NEED FORMATTING, THE PORT NAME MIGHT FULLY CONTAIN THE PATH.
    let mut port = match serialport::new(format!("/dev/{}", port_name), 9600)
        .timeout(Duration::from_millis(2500))
        .open()
    {
        Err(e) => {
            error!("Failed to open port to prandtl controller. Error: {}", e);
            token.cancel();
            return;
        }
        Ok(port) => port,
    };

    loop {
        let client_sensor_data = ClientSensorData {
            pump_speed: crate::models::rpm::Rpm { value: 1000 },
        };

        let packets = read_packets_from_port(&mut port);

        for packet in packets {
            debug!(
                "Control Packet: Type={},Data={},Command={}",
                packet.type_id, packet.data, packet.command
            );
        }

        debug!("Got client sensor data: {}", client_sensor_data);
        if let Err(e) = tx_client_sensor_data.send(client_sensor_data) {
            error!("Failed to send client sensor data. Error: {}", e);
        } else {
            debug!("Sent a client sensor data message.");
        }

        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        };
    }
}

#[instrument(skip_all)]
fn is_ready_to_read_from_port(port: &Box<dyn SerialPort>) -> bool {
    match port.bytes_to_read() {
        Ok(bytes) => {
            trace!("Found {} bytes ready to read from port.", bytes);
            bytes > 0
        }
        Err(e) => {
            warn!(
                "Failed to check if bytes are available to read from port. Error: {}",
                e
            );
            false
        }
    }
}

#[instrument(skip_all)]
fn read_packets_from_port(port: &mut Box<dyn SerialPort>) -> Vec<ControlPacket> {
    if !is_ready_to_read_from_port(port) {
        trace!("Not ready to read yet.");
        return vec![];
    } else {
        trace!("Is ready to read from port.");
    }

    let mut read_buffer: [u8; 1024] = [0; 1024];
    trace!("About to read from port.");
    match port.read(&mut read_buffer) {
        Ok(bytes_read) => {
            trace!("Received {} bytes", bytes_read);
            let (packets, remaining_bytes) =
                decode_packets_from_buffer(&read_buffer[0..bytes_read]);
            debug!(
                "Decoded {} packets from {} bytes with {} left over bytes.",
                packets.len(),
                bytes_read,
                remaining_bytes.len()
            );

            return packets
                .into_iter()
                .map(|raw| ControlPacket {
                    type_id: raw.type_id,
                    data: String::from(raw.data),
                    command: raw.command,
                })
                .collect();
        }
        Err(e) => {
            warn!("Failed to read from port. Error: {}", e);
            return vec![];
        }
    }
}

struct ControlPacket {
    type_id: u8,
    data: String,
    command: bool,
}

// TODO: MOVE THIS TO PROPER PACKET MODEL
#[derive(serde::Serialize, serde::Deserialize)]
struct Packet<'a> {
    type_id: u8,
    data: &'a str,
    command: bool,
}

/// Decode as many packets as possible from a buffer.
/// Returning the vector of packets and any unused bytes from the buffer.
fn decode_packets_from_buffer(buffer: &[u8]) -> (Vec<Packet>, &[u8]) {
    let mut remaining_buffer = buffer;
    let mut packets: Vec<Packet> = vec![];
    while let Ok((packet, extra)) = postcard::take_from_bytes::<Packet>(remaining_buffer) {
        remaining_buffer = extra;
        packets.push(packet);
    }
    (packets, remaining_buffer)
}
