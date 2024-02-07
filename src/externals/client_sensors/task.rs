use anyhow::Result;
use futures::StreamExt;
use serialport::{SerialPort, SerialPortInfo};
use std::time::Duration;
use tokio::{
    select,
    sync::broadcast::{Receiver, Sender},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::models::{
    client_sensor_data::{self, ClientSensorData},
    packet::Packet,
};

/// Try and open communication with a port, send a request communication packet,
/// and receive an accept communication packet response. Returns true if all of these steps
/// pass and false if any of them fail.
async fn try_request_connection_for_port(
    token: CancellationToken,
    port: SerialPortInfo,
) -> (SerialPortInfo, bool) {
    if token.is_cancelled() {
        return (port, false);
    }
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

/// This task handles finding, opening, and sending/receiving packets with
/// the embedded hardware. This task polls to determine when packets are available
/// to read. If not currently reading, it will send packets as they're queued for
/// sending. If communication is lost the task will restart.
#[tracing::instrument(skip_all)]
pub async fn task_handle_client_communication(
    token: CancellationToken,
    tx_packets: Sender<Packet>,
) {
    info!("Started.");

    let port_name = match wait_for_client_port(token.clone()).await {
        Err(e) => {
            warn!("Failed to wait for a client port. Cancelling. Error: {}", e);
            // NOTE: MIGHT NOT NEED THIS CHECK.
            if !token.is_cancelled() {
                token.cancel();
            }
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
        let packets = read_packets_from_port(&mut port);

        for packet in packets {
            debug!("Received Communication Packet: {:?}", packet);

            match tx_packets.send(packet) {
                Err(e) => warn!("Failed to send packet over queue. Error: {}", e),
                Ok(_) => trace!("Successfully sent packet over queue."),
            }
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

/// Listens for incoming client messages. Will convert `ReportSensors` messages
/// into `ClientSensorData` models and transmit them.
#[tracing::instrument(skip_all)]
pub async fn task_process_client_sensor_packets(
    token: CancellationToken,
    tx_client_sensor_data: Sender<ClientSensorData>,
    mut rx_packet: Receiver<Packet>,
) {
    info!("Started.");

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            Ok(data) = rx_packet.recv() => {
                // NOTE: MIGHT BE SUFFICIENT/PREFERRED TO CLONE THE TX SENDER RATHER
                // RATHER THAN SEND A REF.
                handle_report_sensor_packet(data, &tx_client_sensor_data);
            },
        };
    }
}

/// Handle the processing for any incoming client packets.
/// Will only respond to `ReportSensors` type.
/// Will return an error if the `ReportSensors` packet failed to be converted
/// to a `ClientSensorData` or if it failed to be sent over `tx_client_sensor_data`.
/// If it returns an error, the underlying error will be returned.
/// Returns `Ok(())` if either the packet wasn't of type `ReportSensors` or if
/// it was able to successfully generate a `ClientSensorData` and send it.
fn handle_report_sensor_packet(
    packet: Packet,
    tx_client_sensor_data: &Sender<ClientSensorData>,
) -> Result<()> {
    match packet {
        Packet::ReportSensors(packet) => {
            trace!("Received report sensor packet: {:?}", packet);
            let client_sensor_data = match ClientSensorData::try_from(packet) {
                Err(e) => {
                    return Err(e.into());
                }
                Ok(data) => data,
            };

            trace!("Got a client sensor data packet converted.");
            if let Err(e) = tx_client_sensor_data.send(client_sensor_data) {
                return Err(e.into());
            }
            debug!(
                "Sent a client sensor data message. Message: {}",
                client_sensor_data
            );
        }
        _ => { /* NOTE: NOT INTERESTED IN OTHER PACKET TYPES HERE. */ }
    }

    Ok(())
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
fn read_packets_from_port(port: &mut Box<dyn SerialPort>) -> Vec<Packet> {
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

            return packets;
        }
        Err(e) => {
            warn!("Failed to read from port. Error: {}", e);
            return vec![];
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct ControlPacket {
    type_id: u8,
    data: String,
    command: bool,
}

// TODO: MOVE THIS TO PROPER PACKET MODEL
#[derive(serde::Serialize, serde::Deserialize)]
struct PacketLocal<'a> {
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
