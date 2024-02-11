use tokio::sync::broadcast::Receiver;
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

use crate::models::control_event::ControlEvent;

#[tracing::instrument(skip_all)]
pub async fn task_control_event_logging(
    token: CancellationToken,
    mut rx_control_frame: Receiver<ControlEvent>,
) {
    info!("Started.");
    loop {
        tokio::select! {
            _ = token.cancelled() => {
                warn!("Cancelled.");
                break;
            },
            Ok(data) = rx_control_frame.recv() => {
                info!("Got control event: {}", data);
            }
        };
    }
}
