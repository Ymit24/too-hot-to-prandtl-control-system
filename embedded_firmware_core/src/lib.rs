#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;

use bare_metal::CriticalSection;
use common::packet::{Packet, ValveState};
use embedded_hal::{
    blocking::delay::DelayMs,
    digital::v2::{InputPin, OutputPin},
};
use heapless::Vec;
use usbd_serial::{SerialPort, USB_CLASS_CDC};

/// This function determines the valve state from hardware.
pub fn get_valve_state(valve_opened_pin: &dyn InputPin<Error = Infallible>) -> Option<ValveState> {
    match valve_opened_pin.is_high() {
        Ok(is_high) => Some(match is_high {
            true => ValveState::Open,
            false => ValveState::Closed,
        }),
        Err(_) => None,
    }
}

use usb_device::{
    bus::UsbBus,
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};

pub struct Application<'a, B: UsbBus, D: DelayMs<u16>, L: OutputPin> {
    pub serial_port: SerialPort<'a, B>,
    pub usb_device: UsbDevice<'a, B>,
    pub delay: D,
    pub led: L,

    // NOTE: FOR DEBUG SHOULD BE PRIVATE
    /// Represents a queue of packets which have been received.
    pub incoming_packets: Vec<Packet, 16>,

    // NOTE: FOR DEBUG SHOULD BE PRIVATE
    /// Represents a queue of packets which need to be sent.
    pub outgoing_packets: Vec<Packet, 16>,
}

impl<'a, B: UsbBus, D: DelayMs<u16>, L: OutputPin> Application<'a, B, D, L> {
    pub fn new(bus_allocator: &'a UsbBusAllocator<B>, delay: D, led_pin: L) -> Self {
        Self {
            serial_port: SerialPort::new(&bus_allocator),
            usb_device: UsbDeviceBuilder::new(bus_allocator, UsbVidPid(0x2222, 0x3333))
                .manufacturer("LA Tech")
                .product("Too Hot To Prandtl Controller")
                .serial_number("1324")
                .device_class(USB_CLASS_CDC)
                .build(),
            delay,
            led: led_pin,
            incoming_packets: Vec::new(),
            outgoing_packets: Vec::new(),
        }
    }

    /// Poll the USB Device. This should be called from the USB interrupt.
    pub fn poll_usb(&mut self) {
        self.usb_device.poll(&mut [&mut self.serial_port]);
    }

    /// This function will read as many packets from USB as ready.
    /// NOTE: This function MUST be called from a critical section.
    pub fn read_packets_from_usb(&mut self, _cs: &CriticalSection) {
        let mut buffer = [0u8; 128];
        let recv_bytes = match self.serial_port.read(&mut buffer) {
            Err(_) => return,
            Ok(recv_bytes) => recv_bytes,
        };
        if recv_bytes != 0 {
            self.decode_bytes(&buffer[0..recv_bytes]);
        }
    }

    /// Write all outgoing packets to USB. This function ignores write and flush
    /// errors. (Packets may be dropped without warning).
    /// NOTE: This function MUST be called from a critical section.
    pub fn write_packets_to_usb(&mut self, _cs: &CriticalSection) {
        while let Some(packet) = self.outgoing_packets.pop() {
            let buffer: Vec<u8, 128> = postcard::to_vec(&packet).unwrap();
            let _ = self.serial_port.write(&buffer);
        }
        let _ = self.serial_port.flush();
    }

    /// Decode as many packets as available from a buffer.
    /// NOTE: The remaining unused bytes are thrown away.
    /// In the case of strange alignment this COULD POTENTIALLY
    /// drop data or cause corruption.
    /// If the incoming packet vec is full then they will simply be ignored.
    fn decode_bytes(&mut self, buffer: &[u8]) {
        let mut remaining = buffer;
        while let Ok((packet, other)) = postcard::take_from_bytes::<Packet>(remaining) {
            remaining = other;
            let _ = self.incoming_packets.push(packet);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DummyPin<const IS_HIGH: bool>;

    impl<const IS_HIGH: bool> InputPin for DummyPin<IS_HIGH> {
        type Error = Infallible;

        fn is_high(&self) -> Result<bool, Self::Error> {
            Ok(IS_HIGH)
        }

        fn is_low(&self) -> Result<bool, Self::Error> {
            Ok(!IS_HIGH)
        }
    }

    #[test]
    fn get_valve_state_returns_opened() {
        let pin: DummyPin<true> = DummyPin;
        let result = get_valve_state(&pin);
        assert_eq!(result, Some(ValveState::Open));
    }

    #[test]
    fn get_valve_state_returns_closed() {
        let pin: DummyPin<false> = DummyPin;
        let result = get_valve_state(&pin);
        assert_eq!(result, Some(ValveState::Closed));
    }
}
