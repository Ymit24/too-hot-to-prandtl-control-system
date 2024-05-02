use bare_metal::CriticalSection;
use common::{packet::Packet, physical::Rpm};
use embedded_hal::{blocking::delay::DelayMs, digital::v2::OutputPin, Pwm};
use heapless::Vec;
use usb_device::{
    bus::UsbBus,
    class_prelude::UsbBusAllocator,
    device::{UsbDevice, UsbDeviceBuilder, UsbVidPid},
};
use usbd_serial::{SerialPort, USB_CLASS_CDC};

use crate::PrandtlAdc;

pub struct Application<'a, B: UsbBus, D: DelayMs<u16>, L: OutputPin, P1: Pwm, PAdc: PrandtlAdc> {
    pub serial_port: SerialPort<'a, B>,
    pub usb_device: UsbDevice<'a, B>,

    pub delay: D,
    led: L,

    pwm: P1,
    pump_pwm_channel: P1::Channel,
    fan_pwm_channel: P1::Channel,

    padc: PAdc,

    sensor_poll_timer: u8,

    /// Represents a queue of packets which have been received.
    incoming_packets: Vec<Packet, 16>,

    /// Represents a queue of packets which need to be sent.
    outgoing_packets: Vec<Packet, 16>,
}

impl<
        'a,
        B: UsbBus,
        D: DelayMs<u16>,
        L: OutputPin,
        P1: Pwm<Channel = impl Clone, Duty = u32>,
        PAdc: PrandtlAdc,
    > Application<'a, B, D, L, P1, PAdc>
{
    pub fn new(
        bus_allocator: &'a UsbBusAllocator<B>,
        delay: D,
        led_pin: L,
        mut pump_pwm: P1,
        pump_channel: P1::Channel,
        fan_channel: P1::Channel,
        padc: PAdc,
    ) -> Self {
        pump_pwm.enable(pump_channel.clone());
        pump_pwm.enable(fan_channel.clone());

        // Initialize pump and fan to 50%.
        // This should prevent overheating while device boots.
        pump_pwm.set_duty(
            pump_channel.clone(),
            ((pump_pwm.get_max_duty() as f32) * 0.5f32) as u32,
        );
        pump_pwm.set_duty(
            fan_channel.clone(),
            ((pump_pwm.get_max_duty() as f32) * 0.5f32) as u32,
        );

        // TODO: Set valve to PUMP-IN-LOOP
        // TODO: Make sure pump doesn't come on before valve is open.

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
            pwm: pump_pwm,
            pump_pwm_channel: pump_channel,
            fan_pwm_channel: fan_channel,
            padc,
            sensor_poll_timer: 0,
            incoming_packets: Vec::new(),
            outgoing_packets: Vec::new(),
        }
    }

    /// Poll the USB Device. This should be called from the USB interrupt.
    pub fn poll_usb(&mut self) {
        self.usb_device.poll(&mut [&mut self.serial_port]);
    }

    /// The core application loop.
    /// TODO: TEST
    pub fn core_loop(&mut self) {
        self.process_incoming_packets();

        // NOTE: Approximately 0.5Hz.
        //       Consider using hardware timer to schedule reporting sensor data
        self.sensor_poll_timer += 1;
        if self.sensor_poll_timer > 5 {
            self.sensor_poll_timer -= 5;

            self.report_sensors();
        }
    }

    /// Create and push report sensor packet to outgoing packets queue.
    /// NOTE: Consider handling errors
    /// TODO: TEST
    pub fn report_sensors(&mut self) {
        let pump_speed = self.padc.read_pump_sense_norm();
        let fan_speed = self.padc.read_fan_sense_norm();

        // TODO: Refactor this to be cleaner.
        if let Some(pump_speed) = pump_speed {
            if let Some(fan_speed) = fan_speed {
                if let Ok(pump_speed_rpm) = Rpm::new(1800f32, pump_speed) {
                    if let Ok(fan_speed_rpm) = Rpm::new(1800f32, fan_speed) {
                        let _ = self.outgoing_packets.push(Packet::ReportSensors(
                            common::packet::ReportSensorsPacket {
                                pump_speed_norm: pump_speed_rpm,
                                fan_speed_norm: fan_speed_rpm,
                                valve_state: common::packet::ValveState::Open,
                            },
                        ));
                    }
                }
            }
        }
    }

    /// Clear the incoming packet queue and process each packet.
    /// Control packets will trigger changes to the hardware state.
    /// TODO: TEST
    pub fn process_incoming_packets(&mut self) {
        while let Some(packet) = self.incoming_packets.pop() {
            match packet {
                Packet::ReportControlTargets(control_packet) => {
                    let pump_pwm_duty_norm: f32 = control_packet.pump_control_percent.into();
                    let pump_pwm_duty =
                        (pump_pwm_duty_norm * (self.pwm.get_max_duty() as f32)) as u32;

                    self.pwm
                        .set_duty(self.pump_pwm_channel.clone(), pump_pwm_duty);
                    self.pwm
                        .set_duty(self.fan_pwm_channel.clone(), pump_pwm_duty);

                    // TODO: Remove debug indicator
                    let _ = self.led.set_state(control_packet.command.into());
                }
                _ => {}
            }
        }
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
