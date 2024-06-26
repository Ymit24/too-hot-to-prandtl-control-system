use core::{fmt::Display, marker::PhantomData};
use serde::{Deserialize, Serialize};
use thiserror_no_std::Error;

const VALVE_OPEN: (bool, bool) = (true, false);
const VALVE_CLOSED: (bool, bool) = (false, true);

/// Represents the state of the valve. The valve takes multiple seconds to
/// change state and so this allows the control system to avoid rapidly
/// trying to change from open/closed without letting it first finish changing.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Copy)]
pub enum ValveState {
    /// Valve is fully open.
    Open,

    /// Valve is fully closed.
    Closed,

    /// Valve is opening but not fully open.
    Opening,

    /// Valve is closing but not fully closed.
    Closing,

    /// Valve is in an unknown state.
    /// Likely an invalid combination of hi/lo for the sense pins.
    Unknown,
}

impl From<(bool, bool)> for ValveState {
    fn from(value: (bool, bool)) -> Self {
        match value {
            VALVE_OPEN => Self::Open,
            VALVE_CLOSED => Self::Closed,
            _ => Self::Unknown,
        }
    }
}

impl Into<f32> for ValveState {
    fn into(self) -> f32 {
        match self {
            Self::Open | Self::Opening => 1f32,
            Self::Closed | Self::Closing => 0f32,
            _ => 1f32,
        }
    }
}

impl TryFrom<f32> for ValveState {
    type Error = PhantomData<()>;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_sign_negative() || value > 1f32 {
            return Err(PhantomData);
        }

        if value < 0.5f32 {
            Ok(ValveState::Closed)
        } else {
            Ok(ValveState::Open)
        }
    }
}

impl Into<(bool, bool)> for ValveState {
    /// Note: will default to open if in the unknown state
    fn into(self) -> (bool, bool) {
        match self {
            Self::Open | Self::Opening => VALVE_OPEN,
            Self::Closed | Self::Closing => VALVE_CLOSED,
            Self::Unknown => VALVE_OPEN,
        }
    }
}

impl Display for ValveState {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "(ValveState state={:?})", self)
    }
}
