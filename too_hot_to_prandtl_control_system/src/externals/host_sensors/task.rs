use std::time::Duration;

use tokio::sync::broadcast::Sender;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, trace, warn};

use crate::models::host_sensor_data::HostSensorData;

use super::services::HostCpuTemperatureService;

/// Task: Runs periodically to poll host sensors and emit host sensor messages.
/// Can be cancelled.
#[tracing::instrument(skip_all)]
pub async fn task_poll_host_sensors(
    token: CancellationToken,
    service: &impl HostCpuTemperatureService,
    tx_host_sensor_data: Sender<HostSensorData>,
) {
    tracing::info!("Started.");
    loop {
        business_logic(service, &tx_host_sensor_data).await;

        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            _ = tokio::time::sleep(Duration::from_millis(500)) => {}
        };
    }
}

/// Perform task business logic.
/// Poll current host sensor data and try to emit it.
#[tracing::instrument(skip_all)]
async fn business_logic(
    service: &impl HostCpuTemperatureService,
    tx_host_sensor_data: &Sender<HostSensorData>,
) {
    trace!("Executing business logic.");
    let temperature_reading = match service.get_cpu_temp() {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to get cpu temperature. Error: {}", e);
            return;
        }
    };

    debug!("Got cpu temperature: {}", temperature_reading);
    let data = HostSensorData {
        cpu_temperature: temperature_reading,
    };
    if let Err(e) = tx_host_sensor_data.send(data) {
        error!("Failed to broadcast host sensor data. Error: {}", e);
    } else {
        debug!("Sent a host sensor data message.");
    }
}
