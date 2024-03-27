#![cfg_attr(not(test), no_std)]

use bare_metal::CriticalSection;
use common::packet::Packet;
use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin, Pwm};
use heapless::Vec;
use usb_device::{
    bus::UsbBus,
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

pub trait PrandtlAdc {
    fn read_pump_sense_raw(&mut self) -> Option<u16>;
    fn read_fan_sense_raw(&mut self) -> Option<u16>;
}

pub struct Application<
    'a,
    B: UsbBus,
    D: DelayMs<u16>,
    L: OutputPin,
    P1: Pwm,
    P2: Pwm,
    PAdc: PrandtlAdc,
> {
    pub serial_port: SerialPort<'a, B>,
    pub usb_device: UsbDevice<'a, B>,

    pub delay: D,
    pub led: L,

    pub pump_pwm: P1,
    pub fan_pwm: P2,

    pub padc: PAdc,

    // NOTE: FOR DEBUG SHOULD BE PRIVATE
    /// Represents a queue of packets which have been received.
    pub incoming_packets: Vec<Packet, 16>,

    // NOTE: FOR DEBUG SHOULD BE PRIVATE
    /// Represents a queue of packets which need to be sent.
    pub outgoing_packets: Vec<Packet, 16>,
}

impl<
        'a,
        B: UsbBus,
        D: DelayMs<u16>,
        L: OutputPin,
        P1: Pwm<Channel = impl Clone, Duty = u32>,
        P2: Pwm<Channel = impl Clone, Duty = u32>,
        PAdc: PrandtlAdc,
    > Application<'a, B, D, L, P1, P2, PAdc>
{
    pub fn new(
        bus_allocator: &'a UsbBusAllocator<B>,
        delay: D,
        led_pin: L,
        mut pump_pwm: P1,
        mut fan_pwm: P2,
        pump_channel: P1::Channel,
        fan_channel: P2::Channel,
        padc: PAdc,
    ) -> Self {
        pump_pwm.enable(pump_channel.clone());
        fan_pwm.enable(fan_channel.clone());

        pump_pwm.set_duty(pump_channel, pump_pwm.get_max_duty() / 2u32);
        fan_pwm.set_duty(fan_channel, fan_pwm.get_max_duty() / 2);

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
            pump_pwm,
            fan_pwm,
            padc,
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
    /// TODO: TEST
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
    /// TODO: TEST
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
    /// TODO: TEST
    fn decode_bytes(&mut self, buffer: &[u8]) {
        let mut remaining = buffer;
        while let Ok((packet, other)) = postcard::take_from_bytes::<Packet>(remaining) {
            remaining = other;
            let _ = self.incoming_packets.push(packet);
        }
    }
}

#[cfg(test)]
mod tests {}
