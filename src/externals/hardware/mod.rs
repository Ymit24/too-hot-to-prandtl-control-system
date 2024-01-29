use std::{sync::Arc, cell::RefCell};

use self::{
    adapters::{EmitToHardwareAdapter, PollClientSensorAdapter},
    services::{HardwareService, HardwareServiceUsb, HeartbeatMessage},
};

pub mod adapters;
pub mod services;
