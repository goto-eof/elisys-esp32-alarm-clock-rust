mod config;
mod dto;
mod helper;
mod service;

use dto::config_response::Configuration as ConfigurationResponse;
use service::orchestrator_service::orchestrate;

fn main() {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    orchestrate();
}
