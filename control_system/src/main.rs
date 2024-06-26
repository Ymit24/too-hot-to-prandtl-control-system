pub mod models;
pub mod tasks;

pub mod controls;

use anyhow::Result;
use tasks::control_system::task_core_system;
use tasks::host_sensors::{
    services::HostCpuTemperatureServiceActual, task::task_poll_host_sensors,
};
use tokio::{signal, sync::broadcast};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::level_filters::LevelFilter;

use crate::tasks::client_sensors::task::{
    task_handle_client_communication, task_lifetime_management_of_client_communication_task,
    task_process_client_sensor_packets, task_send_control_frames_to_client,
};

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(LevelFilter::TRACE)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    let tracker = TaskTracker::new();

    let token = CancellationToken::new();

    let (tx_client_sensor_data, rx_client_sensor_data) = broadcast::channel(32);
    let (tx_host_sensor_data, rx_host_sensor_data) = broadcast::channel(32);
    let (tx_control_frame, rx_control_frame) = broadcast::channel(32);

    // NOTE: Used to handle packets received from embedded hardware.
    let (tx_packets_from_hw, rx_packets_from_hw) = broadcast::channel(32);

    // NOTE: Used to handle packets to be sent to embedded hardware.
    let (tx_send_packets_to_hw, rx_send_packets_to_hw) = broadcast::channel(32);

    let token_clone = token.clone();
    let tx_control_frame_clone = tx_control_frame.clone();
    tracker.spawn(async {
        task_core_system(
            token_clone,
            rx_client_sensor_data,
            rx_host_sensor_data,
            tx_control_frame_clone,
        )
        .await
    });

    let token_clone = token.clone();
    let host_cpu_service = HostCpuTemperatureServiceActual;
    tracker.spawn(async move {
        task_poll_host_sensors(token_clone, &host_cpu_service, tx_host_sensor_data).await
    });

    let token_clone = token.clone();
    let tx_send_packets_to_hw_clone = tx_send_packets_to_hw.clone();
    tracker.spawn(async {
        task_lifetime_management_of_client_communication_task(
            token_clone,
            tx_packets_from_hw,
            tx_send_packets_to_hw_clone,
        )
        .await;
    });

    let token_clone = token.clone();
    let tx_client_sensor_data_clone = tx_client_sensor_data.clone();
    tracker.spawn(async {
        task_process_client_sensor_packets(
            token_clone,
            tx_client_sensor_data_clone,
            rx_packets_from_hw,
        )
        .await
    });

    let token_clone = token.clone();
    let tx_control_frame_clone = tx_control_frame.clone();
    let rx_control_frame_clone = tx_control_frame_clone.subscribe();
    tracker.spawn(async {
        task_send_control_frames_to_client(
            token_clone,
            rx_control_frame_clone,
            tx_send_packets_to_hw,
        )
        .await
    });

    let token_clone = token.clone();
    let tx_client_sensor_data_clone = tx_client_sensor_data.clone();

    let token_clone = token.clone();

    tokio::select! {
        _ = token_clone.cancelled() => {}
        res = signal::ctrl_c() => {
            match res {
                Ok(_) => {
                    token.cancel();
                },
                Err(e)=>{
                    tracing::error!("Failed to listen for ctrl_c. Error: {}", e);
                    token.cancel();
                }
            };
        },
    }

    tracker.close();
    tracker.wait().await;

    Ok(())
}
