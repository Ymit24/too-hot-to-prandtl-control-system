#![cfg_attr(not(test), no_std)]

pub trait PrandtlAdc {
    fn read_pump_sense_raw(&mut self) -> Option<u16>;
    fn read_fan_sense_raw(&mut self) -> Option<u16>;
}

pub mod application;

#[cfg(test)]
mod tests {}
