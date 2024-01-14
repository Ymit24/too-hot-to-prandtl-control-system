use self::adapters::EmitToLoggingAdapter;

pub mod adapters;

pub struct EventLoggingModule {
    pub control_event_adapter: EmitToLoggingAdapter,
}

impl EventLoggingModule {
    pub fn initialize() -> Self {
        Self {
            control_event_adapter: EmitToLoggingAdapter {},
        }
    }
}
