use crate::app::task_led_commander;
use atsamd_hal::prelude::*;

pub fn led_commander_internal(cx: task_led_commander::Context) {
    let consumer = cx.local.led_commands_consumer;
    let led = cx.local.led;
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
