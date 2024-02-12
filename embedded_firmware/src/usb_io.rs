use atsamd_hal::usb::UsbBus;
use common::packet::Packet;
use heapless::spsc::{Consumer, Producer};
use usbd_serial::SerialPort;

use rtic::mutex_prelude::*;

use crate::app::task_usb_io;

pub fn task_usb_io_internal(cx: task_usb_io::Context) {
    let mut serial = cx.shared.serial;
    let mut tx_led_commands = cx.local.led_commands_producer;
    let rx_packets = cx.local.rx_packets;

    let mut buf = [0u8; 128];
    let bytes = serial.lock(|serial_locked| match serial_locked.read(&mut buf) {
        Err(e) => 0,
        Ok(bytes_read) => bytes_read,
    });
    if bytes != 0 {
        decode_and_process_packets(&buf[0..bytes], &mut tx_led_commands);
    }

    while let Some(packet) = rx_packets.dequeue() {
        let buffer: heapless::Vec<u8, 128> = postcard::to_vec(&packet).unwrap();
        serial.lock(|serial_locked| {
            let _ = serial_locked.write(&buffer);
        });
    }
    serial.lock(|serial_locked| {
        let _ = serial_locked.flush();
    });
}

fn decode_and_process_packets(buffer: &[u8], tx_led_commands: &mut Producer<bool, 16>) {
    let mut remaining = buffer;
    while let Ok((packet, other)) = postcard::take_from_bytes::<Packet>(remaining) {
        remaining = other;
        match packet {
            Packet::ReportControlTargets(packet) => {
                if tx_led_commands.ready() {
                    tx_led_commands.enqueue(packet.command);
                }
            }
            _ => {}
        }
    }
}
