use crate::{
    internals::core::ports::{ControlEventPort, TuningPort},
    models::control_event::ControlEvent,
};

pub struct PollTuningAdapter;
pub struct EmitToReportingAdapter;

impl TuningPort for PollTuningAdapter {
    fn poll_tuning(&self) {
        unimplemented!()
    }
}

impl ControlEventPort for EmitToReportingAdapter {
    fn emit(&self, event: ControlEvent) {
        unimplemented!()
    }
}
