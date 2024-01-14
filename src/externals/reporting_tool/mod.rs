use self::adapters::{EmitToReportingAdapter, PollTuningAdapter};

pub mod adapters;

pub struct ReportingToolModule {
    pub tuning_adapter: PollTuningAdapter,
    pub control_event_adapter: EmitToReportingAdapter,
}

impl ReportingToolModule {
    pub fn initialize() -> Self {
        Self {
            tuning_adapter: PollTuningAdapter {},
            control_event_adapter: EmitToReportingAdapter {},
        }
    }
}
