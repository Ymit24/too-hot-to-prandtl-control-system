#![cfg_attr(not(test), no_std)]

use core::convert::Infallible;

use common::packet::ValveState;
use embedded_hal::digital::v2::InputPin;

/// This function determines the valve state from hardware.
pub fn get_valve_state(
    valve_opened_pin: &dyn InputPin<Error = Infallible>,
) -> Option<ValveState> {
    match valve_opened_pin.is_high() {
        Ok(is_high) => Some(match is_high {
            true => ValveState::Open,
            false => ValveState::Closed,
        }),
        Err(_) => None,
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
