use tokio::sync::broadcast::{Receiver, Sender};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::{
    controls::generate_control_frame,
    models::{
        client_sensor_data::ClientSensorData, control_event::ControlEvent,
        host_sensor_data::HostSensorData,
    },
};

/// Task: Activate when a host or client sensor data is emitted.
/// Generate a control frame when both a client and host data have been
/// emitted which is updated everytime a host or client data are emitted.
/// Can be cancelled.
#[tracing::instrument(skip_all)]
pub async fn task_core_system(
    token: CancellationToken,
    mut rx_client_sensor_data: Receiver<ClientSensorData>,
    mut rx_host_sensor_data: Receiver<HostSensorData>,
    tx_control_frame: Sender<ControlEvent>,
) {
    info!("Started.");

    let mut current_host_frame: Option<HostSensorData> = None;
    let mut current_client_frame: Option<ClientSensorData> = None;

    loop {
        business_logic(current_client_frame, current_host_frame, &tx_control_frame).await;

        tokio::select! {
            _ = token.cancelled() => {
                warn!("Canceled.");
                break;
            },
            Ok(data) = rx_client_sensor_data.recv() => {
                current_client_frame = Some(data);
                trace!("Received client frame.");
            },
            Ok(data) = rx_host_sensor_data.recv() => {
                current_host_frame = Some(data);
                trace!("Received host frame.");
            }
        }
    }
}

/// Perform task business logic. If both host and client data are available,
/// generate a control frame and try to emit it.
#[tracing::instrument(skip_all)]
async fn business_logic(
    current_client_frame: Option<ClientSensorData>,
    current_host_frame: Option<HostSensorData>,
    tx_control_frame: &Sender<ControlEvent>,
) {
    trace!("Executing business logic.");
    if let Some(client) = current_client_frame {
        if let Some(host) = current_host_frame {
            let control_event = generate_control_frame(client, host);
            if let Err(e) = tx_control_frame.send(control_event) {
                error!("Failed to broadcast control frame. Error: {}", e);
            } else {
                debug!("Sent a control frame.");
            }
        }
    }
}
