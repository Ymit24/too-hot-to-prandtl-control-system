use common::packet::{Packet, ReportSensorsPacket, ValveState};
use embedded_hal::adc::OneShot;
use heapless::spsc::Producer;

use crate::app::blink;

pub fn blink_internal(cx: blink::Context) {
        let tx_packets = cx.local.tx_packets;
    let adc = cx.local.adc_a5;
    let mut a0 = cx.local.a0;

    let data: u16 = adc.read(a0).unwrap();

    let pack = Packet::ReportSensors(ReportSensorsPacket {
        fan_speed_norm: data,
        pump_speed_norm: 200,
        valve_state: ValveState::Opening,
    });

    if tx_packets.ready() {
        let _ = tx_packets.enqueue(pack.clone());
    }
}
