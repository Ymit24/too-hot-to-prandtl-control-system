use externals::{
    event_logging::{adapters::EmitToLoggingAdapter, EventLoggingModule},
    hardware::{
        adapters::{EmitToHardwareAdapter, PollClientSensorAdapter},
        HardwareModule,
    },
    host_sensors::HostSensorModule,
};
use internals::core::{ports::ClientSensorPort, system::CoreSystem};

use crate::{
    externals::{
        host::host_sensor_adapter::HostSensorAdapter, reporting_tool::ReportingToolModule,
    },
    internals::control_system::{app::Application, models::Temperature},
};

pub mod externals;
pub mod internals;
pub mod models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let raw_temp = 90f32;
    let _cpu_temp = Temperature::try_from(raw_temp)?;

    let hsp = HostSensorAdapter {};
    let app = Application::new(hsp);

    app.run()?;

    let HardwareModule {
        client_sensor_adapter,
        control_event_adapter: emit_to_hardware_adapter,
    } = HardwareModule::initialize();

    let EventLoggingModule {
        control_event_adapter: emit_to_logging_adapter,
    } = EventLoggingModule::initialize();

    let HostSensorModule {
        host_sensor_adapter,
    } = HostSensorModule::initialize();

    let ReportingToolModule {
        tuning_adapter,
        control_event_adapter: emit_to_reporting_tool,
    } = ReportingToolModule::initialize();

    let _core = CoreSystem::new(
        client_sensor_adapter,
        host_sensor_adapter,
        tuning_adapter,
        vec![
            &emit_to_hardware_adapter,
            &emit_to_logging_adapter,
            &emit_to_reporting_tool,
        ],
    );

    Ok(())
}
