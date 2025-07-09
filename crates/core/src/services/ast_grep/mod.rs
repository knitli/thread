mod agproject;
mod agconfig;

use tower_service::Service;
use std::path::{Path, PathBuf};
pub use crate::{
    agconfig::{AstGrepConfig, ConfigError, TraceError},
    agproject::{ProjectInfo, DiscoveryError},
};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone)]
pub struct ConfigRequest {
    pub path: Option<PathBuf>,
}

pub trait ConfigReader: Service<ConfigRequest, Response = Option<AstGrepConfig>, Error = ConfigError> {}

#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
#[derive(Debug, Clone)]
pub enum TraceEvent {
    ProjectInfo { is_project: bool, dir: Option<PathBuf> },
    FileScan { path: PathBuf, lang: ThreadLang },
    RuleStats { effective: usize, skipped: usize },
    FileStats { scanned: usize, skipped: usize },
}

pub trait TraceOutput: Service<TraceEvent, Response = (), Error = TraceError> {}
