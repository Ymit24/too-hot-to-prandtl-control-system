use core::convert::Infallible;

use crate::app::task_led_commander;
use embedded_hal as ehal;
use heapless::spsc::Consumer;

pub fn led_commander_internal(cx: task_led_commander::Context) {
    let consumer = cx.local.led_commands_consumer;
    let led = cx.local.led;
    led_commander_internal2(led, consumer);
}

pub fn led_commander_internal2<LED: ehal::digital::v2::OutputPin<Error = Infallible>>(
    led: &mut LED,
    consumer: &mut Consumer<bool, 16>,
) {
    if consumer.ready() {
        while let Some(command) = consumer.dequeue() {
            if command {
                led.set_high();
            } else {
                led.set_low();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use heapless::spsc::Queue;

    use super::*;

    struct FakeLedOutput {
        state: bool,
    }

    impl ehal::digital::v2::OutputPin for FakeLedOutput {
        type Error = Infallible;

        fn set_low(&mut self) -> Result<(), Self::Error> {
            self.state = false;
            Ok(())
        }

        fn set_high(&mut self) -> Result<(), Self::Error> {
            self.state = true;
            Ok(())
        }
    }

    #[test]
    fn test_led_commander() {
        let mut fake_led = FakeLedOutput { state: false };
        let mut queue: Queue<bool, 16> = Queue::new();
        let (_, mut consumer) = queue.split();
        led_commander_internal2(&mut fake_led, &mut consumer);
    }
}
