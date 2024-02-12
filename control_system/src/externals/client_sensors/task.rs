use anyhow::Result;
use futures::StreamExt;
use serialport::{SerialPort, SerialPortInfo};
use std::{fmt::write, time::Duration};
use tokio::{
    select,
    sync::broadcast::{Receiver, Sender},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{debug, error, info, instrument, trace, warn};

use crate::models::{
    client_sensor_data::{self, ClientSensorData},
    control_event::ControlEvent,
};

use common::packet::*;

const PRODUCT_NAME: &str = "Too Hot To Prandtl Controller";
const SERIAL_NUMBER: &str = "1324";

/// Check if a port is for the embedded hardware.
/// Checks both the serial number and product name of the port.
#[instrument(skip_all)]
fn is_port_for_embedded_hardware(token: CancellationToken, port: SerialPortInfo) -> bool {
    if token.is_cancelled() {
        warn!("Trying to request connection for a port but the token is cancelled. Aborting.");
        return false;
    }
    trace!("Checking port '{}'.", port.port_name);

    match port.port_type {
        serialport::SerialPortType::UsbPort(usb_info) => {
            if let Some(serial_number) = usb_info.serial_number {
                if serial_number != SERIAL_NUMBER {
                    debug!("Wrong serial number!");
                    return false;
                }
            } else {
                debug!("Failed to get serial number from port.");
                return false;
            }
            if let Some(product_name) = usb_info.product {
                if product_name != PRODUCT_NAME {
                    debug!("Wrong product name!");
                    return false;
                }
            } else {
                debug!("Failed to get product name from port.");
                return false;
            }
        }
        _ => {
            debug!("Wrong port type.");
            return false;
        }
    }
    debug!("This port is the correct client port.");
    true
}

#[instrument(skip_all)]
fn find_client_port(token: CancellationToken) -> Option<SerialPortInfo> {
    let ports = match serialport::available_ports() {
        Err(e) => {
            error!("Failed to get any ports! Error: {}", e);
            return None;
        }
        Ok(ports) => ports,
    };

    trace!("Found {} ports to check.", ports.len());

    ports
        .into_iter()
        .filter_map(|port| {
            if is_port_for_embedded_hardware(token.clone(), port.clone()) {
                Some(port)
            } else {
                None
            }
        })
        .collect::<Vec<SerialPortInfo>>()
        .first()
        .map(|x| x.clone())
}

#[instrument(skip_all)]
async fn wait_for_client_port(token: CancellationToken) -> Result<SerialPortInfo, String> {
    loop {
        if token.is_cancelled() {
            warn!("Token was cancelled.");
            return Err("Cancelled".into());
        }
        trace!("Looking for client port.");
        if let Some(port_name) = find_client_port(token.clone()) {
            return Ok(port_name);
        }
        trace!("Sleeping briefly before checking again.");
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
}

pub async fn task_lifetime_management_of_client_communication_task(
    token: CancellationToken,
    tx_packets_from_hw: Sender<Packet>,
    tx_packets_to_hw: Sender<Packet>,
) {
    info!("Started");

    loop {
        debug!("About to start client communication task.");
        let tx_packets_from_hw_clone = tx_packets_from_hw.clone();
        task_handle_client_communication(
            token.clone(),
            tx_packets_from_hw_clone.clone(),
            tx_packets_to_hw.subscribe(),
        )
        .await;
        warn!("Client communication task exited.");

        if token.is_cancelled() {
            warn!("Cancelled.");
            break;
        }
        info!("Restarting client communication task.");
    }
}

/// This task handles finding, opening, and sending/receiving packets with
/// the embedded hardware. This task polls to determine when packets are available
/// to read. If not currently reading, it will send packets as they're queued for
/// sending. If communication is lost the task will restart.
#[tracing::instrument(skip_all)]
pub async fn task_handle_client_communication(
    token: CancellationToken,
    tx_packets_from_hw: Sender<Packet>,
    mut rx_packets_to_hw: Receiver<Packet>,
) {
    info!("Started.");

    trace!("Waiting on client port to be identified.");
    let port_info = match wait_for_client_port(token.clone()).await {
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
    info!("Found a client port! Name: {}", port_info.port_name);

    let mut port = match serialport::new(port_info.port_name, 9600)
        .timeout(Duration::from_millis(1000))
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
        let packets = match read_packets_from_port(&mut port) {
            Ok(packets) => packets,
            Err(e) => {
                error!("Failed to read packets from port. Error: {}", e);
                break;
            }
        };

        for packet in packets {
            debug!("Received Communication Packet: {:?}", packet);

            match tx_packets_from_hw.send(packet) {
                Err(e) => warn!("Failed to send packet over queue. Error: {}", e),
                Ok(_) => trace!("Successfully sent packet over queue."),
            }
        }

        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            Ok(data) = rx_packets_to_hw.recv() => {
                debug!("Received packet to write to port. Packet: {:?}",data);
                // NOTE: Received a packet TO SEND to hw
                if let Err(e) = write_packet_to_port(&mut port, data) {
                    warn!("Failed to write packet to port! Error: {}", e);
                } else {
                    debug!("Successfully wrote packet to port!");
                }
            },
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        };
    }
}

/// Send a single packet of data to the embedded hardware.
#[instrument(skip_all)]
fn write_packet_to_port(port: &mut Box<dyn SerialPort>, packet: Packet) -> Result<usize> {
    match postcard::to_vec::<Packet, 64>(&packet) {
        Err(e) => {
            warn!("Failed to encode packet to byte array. Error: {}", e);
            Err(e.into())
        }
        Ok(buffer) => match port.write(buffer.as_slice()) {
            Err(e) => {
                error!("Failed to write byte buffer to port. Error: {}", e);
                Err(e.into())
            }
            Ok(length) => {
                debug!("Successfully wrote {} bytes to port.", length);
                Ok(length)
            }
        },
    }
}

/// Listens for incoming client messages. Will convert `ReportSensors` messages
/// into `ClientSensorData` models and transmit them.
#[tracing::instrument(skip_all)]
pub async fn task_process_client_sensor_packets(
    token: CancellationToken,
    tx_client_sensor_data: Sender<ClientSensorData>,
    mut rx_packets_from_hw: Receiver<Packet>,
) {
    info!("Started.");

    loop {
        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            Ok(data) = rx_packets_from_hw.recv() => {
                debug!("Got packet from hardware. Packet: {:?}",data);
                // NOTE: MIGHT BE SUFFICIENT/PREFERRED TO CLONE THE TX SENDER RATHER
                // RATHER THAN SEND A REF.
                if let Err(e) = handle_report_sensor_packet(data, &tx_client_sensor_data) {
                    error!("Failed to handle report sensor packet. Error: {}", e);
                } else {
                    debug!("Successfully handled report sensor packet.");
                }
            },
        };
    }
}

/// This task will convert control frames into packets and queue them for
/// transmission to the embedded hardware.
#[instrument(skip_all)]
pub async fn task_send_control_frames_to_client(
    token: CancellationToken,
    mut rx_control_frame: Receiver<ControlEvent>,
    tx_send_packets_to_hw: Sender<Packet>,
) {
    info!("Started");
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            Ok(data) = rx_control_frame.recv() => {
                match convert_control_frame_to_packet_and_send_to_hardware(data, &tx_send_packets_to_hw) {
                    Err(e) => {
                        error!("Failed to packetize and queue control frame for transmission. Error: {}", e);
                    },
                    Ok(_) => {
                        debug!("Successfully packetized and queued control frame for transmission.");
                    }
                }
            },
        };
    }
}

/// Convert a control frame into a packet and queue it to be sent.
/// Returns a result, ```Ok(())``` if the packet was converted and queued,
/// ```Err``` otherwise.
fn convert_control_frame_to_packet_and_send_to_hardware(
    control_frame: ControlEvent,
    tx_send_packets_to_hw: &Sender<Packet>,
) -> Result<()> {
    let packet = match Packet::try_from(control_frame) {
        Err(e) => {
            return Err(e.into());
        }
        Ok(packet) => packet,
    };
    match tx_send_packets_to_hw.send(packet) {
        Err(e) => Err(e.into()),
        Ok(_) => Ok(()),
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
        _ => {
            /* NOTE: NOT INTERESTED IN OTHER PACKET TYPES HERE. */
            trace!("Received packet other than sensor packet.");
        }
    }

    Ok(())
}

#[instrument(skip_all)]
fn is_ready_to_read_from_port(port: &Box<dyn SerialPort>) -> Result<bool> {
    match port.bytes_to_read() {
        Ok(bytes) => {
            trace!("Found {} bytes ready to read from port.", bytes);
            Ok(bytes > 0)
        }
        Err(e) => {
            warn!(
                "Failed to check if bytes are available to read from port. Error: {}",
                e
            );
            Err(e.into())
        }
    }
}

#[instrument(skip_all)]
fn read_packets_from_port(port: &mut Box<dyn SerialPort>) -> Result<Vec<Packet>> {
    match is_ready_to_read_from_port(port) {
        Ok(true) => {
            trace!("Is ready to read from port.");
        }
        Ok(false) => {
            trace!("Not ready to read yet.");
            return Ok(vec![]);
        }
        Err(e) => {
            trace!("Not ready to read yet with error. Error: {}", e);
            return Err(e.into());
        }
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

            return Ok(packets);
        }
        Err(e) => {
            warn!("Failed to read from port. Error: {}", e);
            return Err(e.into());
        }
    }
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
    if buffer.len() > 0 && packets.is_empty() {
        warn!("Didn't decode a single packet from {} bytes!", buffer.len());
    }
    (packets, remaining_buffer)
}
