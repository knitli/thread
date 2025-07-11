// SPDX-FileCopyrightText: 2025 Herrington Darkholme <2883231+HerringtonDarkholme@users.noreply.github.com>
// SPDX-FileCopyrightText: 2025 Knitli Inc. <knitli@knit.li>
// SPDX-FileContributor: Adam Poulemanos <adam@knit.li>
//
// SPDX-License-Identifier: MIT

use super::{CustomLang, LanguageGlobs, SerializableInjection, ThreadLang};
use ag_service_rule::{
    DeserializeEnv, error_context::ErrorContext as EC, GlobalRules, RuleCollection, RuleConfig, RuleOverwrite,
    RuleTrace, from_str, from_yaml_string,
};
use thread_core::services::ast_grep::;

use anyhow::{Context, Result};

use crate::config_file_type;
use ignore::WalkBuilder;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use thread_utils::FastMap;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Clone)]
pub struct TestConfig {
    pub test_dir: PathBuf,
    /// Specify the directory containing snapshots. The path is relative to `test_dir`
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub snapshot_dir: Option<PathBuf>,
}

impl From<PathBuf> for TestConfig {
    fn from(path: PathBuf) -> Self {
        TestConfig {
            test_dir: path,
            snapshot_dir: None,
        }
    }
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Clone)]
pub struct AstGrepConfig {
    /// YAML rule directories
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub rule_dirs: Vec<PathBuf>,
    /// test configurations
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub test_configs: Option<Vec<TestConfig>>,
    /// util rules directories
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub util_dirs: Option<Vec<PathBuf>>,
    /// configuration for custom languages
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub custom_languages: Option<FastMap<String, CustomLang>>,
    /// additional file globs for languages
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub language_globs: Option<LanguageGlobs>,
    /// injection config for embedded languages
    #[cfg_attr(
        feature = "serde",
        serde(default, skip_serializing_if = "Vec::is_empty")
    )]
    pub language_injections: Vec<SerializableInjection>,
}

#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(rename_all = "camelCase")
)]
#[derive(Clone)]
pub struct ThreadProjectConfig<C, D, R> {
    config_reader: C,
    discovery: D,
    rule_loader: R,
    project_dir: PathBuf,
    rule_dirs: Vec<PathBuf>,
    test_configs: Option<Vec<TestConfig>>,
    util_dirs: Option<Vec<PathBuf>>,
}

impl<C, D, R> ThreadProjectConfig<C, D, R>
where
    C: ConfigReader,
    D: ProjectDiscovery,
    R: RuleLoader,
{
    pub async fn setup(
        config_path: Option<PathBuf>,
        mut config_reader: C,
        mut discovery: D,
        rule_loader: R,
    ) -> Result<Result<Self>> {
        // Discover project using the discovery service
        let discovery_req = DiscoveryRequest {
            start_path: config_path
                .clone()
                .unwrap_or_else(|| std::env::current_dir().unwrap()),
            config_name: "sgconfig.yml".to_string(),
        };

        let Some(project_info) = discovery.call(discovery_req).await? else {
            return Ok(Err(anyhow::anyhow!("Project not found")));
        };

        // Read config using the config service
        let config_req = ConfigRequest { path: config_path };
        let Some(mut thread_config) = config_reader.call(config_req).await? else {
            return Ok(Err(anyhow::anyhow!("Config not found")));
        };

        let config = ThreadProjectConfig {
            config_reader,
            discovery,
            rule_loader,
            project_dir: project_info.project_dir,
            rule_dirs: thread_config.rule_dirs.drain(..).collect(),
            test_configs: thread_config.test_configs.take(),
            util_dirs: thread_config.util_dirs.take(),
        };

        #[cfg(feature = "ag-dynamic-language")]
        register_custom_language(&config.project_dir, thread_config)?;

        Ok(Ok(config))
    }

    pub async fn find_rules(
        &mut self,
        rule_overwrite: RuleOverwrite,
    ) -> Result<(RuleCollection<ThreadLang>, RuleTrace)> {
        let rule_req = RuleLoadRequest {
            rule_dirs: self.rule_dirs.clone(),
            util_dirs: self.util_dirs.clone(),
            base_dir: self.project_dir.clone(),
        };

        self.rule_loader.call(rule_req).await
    }
}
