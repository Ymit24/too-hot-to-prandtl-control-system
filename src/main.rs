pub mod externals;
pub mod models;

pub mod controls;
pub mod system;

use anyhow::Result;
use system::task_core_system;
use tokio::{signal, sync::broadcast};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> Result<()> {
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_max_level(LevelFilter::DEBUG)
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;
    let tracker = TaskTracker::new();

    let token = CancellationToken::new();

    let (_tx_client_sensor_data, rx_client_sensor_data) = broadcast::channel(32);
    let (_tx_host_sensor_data, rx_host_sensor_data) = broadcast::channel(32);
    let (tx_control_frame, _rx_control_frame) = broadcast::channel(32);

    let token_clone = token.clone();
    tracker.spawn(async {
        task_core_system(
            token_clone,
            rx_client_sensor_data,
            rx_host_sensor_data,
            tx_control_frame,
        )
        .await
    });

    if let Err(e) = signal::ctrl_c().await {
        tracing::error!("Failed to listen for ctrl_c. Error: {}", e);
    }

    token.cancel();
    tracker.close();
    tracker.wait().await;

    Ok(())
}
