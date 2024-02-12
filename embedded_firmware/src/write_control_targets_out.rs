use atsamd_hal::pwm::Channel;
use embedded_hal::{Pwm};

use crate::app::task_write_control_targets_out;

pub fn task_write_control_targets_out_internal(cx: task_write_control_targets_out::Context) {
    let pump_pwm = cx.local.pump_pwm;
    let rx_control_frames = cx.local.rx_control_frames;

    while let Some(control_frame) = rx_control_frames.dequeue() {
        let norm = (control_frame.pump_control_voltage as f32) / u8::max_value() as f32;
        pump_pwm.set_duty(Channel::_0, ((pump_pwm.get_max_duty() as f32) * norm) as u32);
    }
}
