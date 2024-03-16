use atsamd_hal::pwm::Channel;
use embedded_hal::Pwm;

use crate::app::task_write_control_targets_out;

pub fn task_write_control_targets_out_internal(cx: task_write_control_targets_out::Context) {
    let pump_pwm = cx.local.pump_pwm;
    let fan_pwm = cx.local.fan_pwm;
    let rx_control_frames = cx.local.rx_control_frames;

    while let Some(control_frame) = rx_control_frames.dequeue() {
        pump_pwm.set_duty(
            Channel::_1,
            control_frame.pump_control_pwm
        );
        fan_pwm.set_duty(
            Channel::_1,
            control_frame.fan_control_pwm
        );
    }
}
