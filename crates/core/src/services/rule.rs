use std::path::PathBuf;
use tower_service::Service;
use thread_languages::{RuleCollection, ThreadLang, RuleTrace, RuleError};

#[derive(Debug, Clone)]
pub struct RuleLoadRequest {
    pub rule_dirs: Vec<PathBuf>,
    pub util_dirs: Option<Vec<PathBuf>>,
    pub base_dir: PathBuf,
}

pub trait RuleLoader: Service<RuleLoadRequest, Response = (RuleCollection<ThreadLang>, RuleTrace), Error = RuleError> {}
