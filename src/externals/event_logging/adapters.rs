use crate::{internals::core::ports::ControlEventPort, models::control_event::ControlEvent};

pub struct EmitToLoggingAdapter;

impl ControlEventPort for EmitToLoggingAdapter {
    fn emit(&self, event: ControlEvent) {
        println!("[EVENT LOGGING] Control Event: {}", event);
    }
}
