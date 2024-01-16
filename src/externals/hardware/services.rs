use serde_derive::Serialize;
use serialport::SerialPort;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};
use thiserror::Error;

pub trait HardwareService {
    fn queue_message(&mut self, msg: &dyn HardwareMessage);
    fn dequeue_messages_by_id(&mut self, mid: u16) -> Vec<Box<dyn HardwareMessage>>;
    fn poll(&mut self);
}

pub trait HardwareMessage: Send {
    fn get_id(&self) -> u16;
    fn serialize(&self) -> &[u8];
}

// example message
#[derive(Serialize)]
pub struct HeartbeatMessage {
    pub mid: u16,
}

// example message
#[derive(Serialize)]
pub struct ControlMessage {
    pub mid: u16,
    pub value: f32,
}

impl ControlMessage {
    pub fn new(value: f32) -> Self {
        Self { mid: 2, value }
    }
}

impl HardwareMessage for ControlMessage {
    fn get_id(&self) -> u16 {
        self.mid
    }

    fn serialize(&self) -> &[u8] {
        bincode::serialize(&self)
            .expect("Failed to serialize control message!")
            .as_slice()
    }
}

impl HeartbeatMessage {
    pub fn new() -> Self {
        Self { mid: 1 }
    }
}

impl HardwareMessage for HeartbeatMessage {
    fn get_id(&self) -> u16 {
        self.mid
    }

    fn serialize(&self) -> &[u8] {
        bincode::serialize(&self)
            .expect("Failed to serialize heartbeat message!")
            .as_slice()
    }
}

pub struct HardwareServiceUsb {
    port: Box<dyn SerialPort>,
    communication: HardwareCommunication,
}

#[derive(Error, Debug)]
pub enum HardwareServiceError {
    #[error("Failed to open serial port. Device might be disconnected")]
    FailedToOpenPort,
}

impl HardwareServiceUsb {
    // NOTE: I DON'T LOVE HAVING THIS CONST
    pub fn new() -> Self {
        let port = serialport::new("/dev/ttyACM0", 57_000)
            .timeout(Duration::from_millis(2500))
            .open()
            .map_err(|_x| HardwareServiceError::FailedToOpenPort)
            .expect("Failed to open port!");

        let (s, r) = mpsc::channel::<Box<dyn HardwareMessage>>();

        s.send(Box::new(HeartbeatMessage::new()));
        s.send(Box::new(ControlMessage::new(2f32)));

        let asdf = r.recv().unwrap();

        let mut communication = HardwareCommunication::new();
        communication.start();

        Ok(Self {
            port,
            communication,
        })
    }
}

impl HardwareService for HardwareServiceUsb {
    fn queue_message(&mut self, msg: &dyn HardwareMessage) {
        // bincode::serialize_into(&mut self.port, &msg).expect("Failed to send message");
    }

    fn poll(&mut self) {
        unimplemented!()
    }

    fn dequeue_messages_by_id(&mut self, mid: u16) -> Vec<Box<dyn HardwareMessage>> {
        unimplemented!()
    }
}

struct HardwareCommunication {
    handle: Option<JoinHandle<()>>,
    running: Arc<AtomicBool>,
}

impl HardwareCommunication {
    pub fn new() -> Self {
        Self {
            handle: None,
            running: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start(&mut self) {
        self.running.store(true, Ordering::SeqCst);
        let running = self.running.clone();

        self.handle = Some(thread::spawn(move || {
            while running.load(Ordering::SeqCst) {
                // read / write here
                thread::sleep(Duration::from_millis(500));
            }
        }));
    }
}

impl Drop for HardwareCommunication {
    fn drop(&mut self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
        self.handle
            .take()
            .expect("Failed to stop non-running thread")
            .join()
            .expect("Failed to stop thread");
    }
}
