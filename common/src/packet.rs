use fixedstr::str8;
use serde::{Deserialize, Serialize};
use crate::physical::{Percentage, Rpm, ValveState};

// TODO: Impl Display for Packet

/// Used to communicate with embedded hardware.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum Packet {
    RequestConnection(RequestConnectionPacket),
    AcceptConnection(AcceptConnectionPacket),
    ReportSensors(ReportSensorsPacket),
    ReportControlTargets(ReportControlTargetsPacket),
    ReportLogLine(ReportLogLinePacket),
}

/// Represents a request to establish connection. Used to determine
/// which port the embedded hardware is plugged into.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct RequestConnectionPacket {
    special_pattern: [u8; 8],
}

/// Represents a response from embedded hardware. Used to determine
/// which port it was plugged into.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct AcceptConnectionPacket {
    special_pattern: [u8; 8],
}

/// Represents a snapshot of normalized sensor data from the embedded hardware.
/// Used for processing into an input into the control system. Will need to be
/// processed into physical unit representation.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ReportSensorsPacket {
    /// Normalized representation of the fan's rpm.
    pub fan_speed_rpm: Rpm,

    /// Normalized representation of the pump's rpm.
    pub pump_speed_rpm: Rpm,

    /// Valve State
    pub valve_state: ValveState,
}

/// Represents a snapshot of raw target control state. Sent from the host
/// to the embedded hardware.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ReportControlTargetsPacket {
    /// The voltage value which the embedded hardware should immediately output
    /// for the fan.
    pub fan_control_percent: Percentage,

    /// The voltage value which the embedded hardware should immediately output
    /// for the pump.
    pub pump_control_percent: Percentage,

    /// The valve is either instructed to begin opening or closing.
    /// Sending the state which the valve is in results in nothing happening.
    pub valve_control_state: ValveState,
}

/// Represents a diagnostic log line from the embedded hardware.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct ReportLogLinePacket {
    pub log_line: str8,
}

impl RequestConnectionPacket {
    /// Used to create an instance of this struct.
    /// Sets the `special_pattern` to a known value.
    pub fn new() -> Self {
        Self {
            // TODO: DOUBLE CHECK THIS (is *b"..." okay)
            special_pattern: *b"ab2dwask",
        }
    }

    /// Used to create a new instance of this struct wrapped in a packet.
    /// Typically what will be used.
    pub fn new_packet() -> Packet {
        Packet::RequestConnection(Self::new())
    }
}
