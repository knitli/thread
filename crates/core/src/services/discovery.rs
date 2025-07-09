use std::path::PathBuf;
use tower_service::Service;
use thread_languages::ThreadLang;

#[derive(Debug, Clone)]
pub struct DiscoveryRequest {
    pub start_path: PathBuf,
    pub config_name: String,
}

pub trait ProjectDiscovery: Service<DiscoveryRequest, Response = Option<ProjectInfo>, Error = DiscoveryError> {}
