use common::packet::{Packet, ReportSensorsPacket, ValveState};
use heapless::spsc::Producer;

pub fn blink_internal(mut tx_packets: &mut Producer<Packet, 16>) {
    let pack = Packet::ReportSensors(ReportSensorsPacket {
        fan_speed_norm: 100,
        pump_speed_norm: 200,
        valve_state: ValveState::Opening,
    });

    if tx_packets.ready() {
        let _ = tx_packets.enqueue(pack.clone());
    }
}
