use externals::{
    event_logging::EventLoggingModule, hardware::HardwareModule, host_sensors::HostSensorModule,
    reporting_tool::ReportingToolModule,
};
use internals::core::system::CoreSystem;

pub mod externals;
pub mod internals;
pub mod models;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
