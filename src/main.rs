use crate::{
    externals::host::host_sensor_adapter::HostSensorAdapter,
    internals::control_system::{app::Application, models::Temperature},
};

pub mod externals;
pub mod internals;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_temp = 90f32;
    let _cpu_temp = Temperature::try_from(raw_temp)?;

    let hsp = HostSensorAdapter {};
    let app = Application::new(hsp);

    app.run()?;

    Ok(())
}
