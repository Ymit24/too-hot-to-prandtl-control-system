use serialport::SerialPort;
use std::time::Duration;
use tokio::sync::broadcast::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::models::client_sensor_data::ClientSensorData;

#[tracing::instrument(skip_all)]
pub async fn task_poll_client_sensors(
    token: CancellationToken,
    tx_client_sensor_data: Sender<ClientSensorData>,
) {
    info!("Started.");

    // TODO: DON'T HARDCODE THE PORT
    let mut port = match serialport::new("/dev/ttyACM3", 9600)
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
