use crate::prelude::*;

use crate::setup::{CombinedState, ResourceSetupChange, ResourceSetupInfo, SetupChangeType};
use serde::{Deserialize, Serialize};

pub fn default_tracking_table_name(flow_name: &str) -> String {
    format!(
        "{}__cocoindex_tracking",
        utils::db::sanitize_identifier(flow_name)
    )
}

pub fn default_source_state_table_name(flow_name: &str) -> String {
    format!(
        "{}__cocoindex_srcstate",
        utils::db::sanitize_identifier(flow_name)
    )
}

pub const CURRENT_TRACKING_TABLE_VERSION: i32 = 1;



#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TrackingTableSetupState {
    pub table_name: String,
    pub version_id: i32,
    #[serde(default)]
    pub source_state_table_name: Option<String>,
    #[serde(default)]
    pub has_fast_fingerprint_column: bool,
}

#[derive(Debug)]
pub struct TrackingTableSetupChange {
    pub desired_state: Option<TrackingTableSetupState>,

    pub min_existing_version_id: Option<i32>,
    pub legacy_tracking_table_names: BTreeSet<String>,

    pub source_state_table_always_exists: bool,
    pub legacy_source_state_table_names: BTreeSet<String>,

    pub source_names_need_state_cleanup: BTreeMap<i32, BTreeSet<String>>,

    has_state_change: bool,
}

impl TrackingTableSetupChange {
    pub fn new(
        desired: Option<&TrackingTableSetupState>,
        existing: &CombinedState<TrackingTableSetupState>,
        source_names_need_state_cleanup: BTreeMap<i32, BTreeSet<String>>,
    ) -> Option<Self> {
        let legacy_tracking_table_names = existing
            .legacy_values(desired, |v| &v.table_name)
            .into_iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let legacy_source_state_table_names = existing
            .legacy_values(desired, |v| &v.source_state_table_name)
            .into_iter()
            .filter_map(|v| v.clone())
            .collect::<BTreeSet<_>>();
        let min_existing_version_id = existing
            .always_exists()
            .then(|| existing.possible_versions().map(|v| v.version_id).min())
            .flatten();
        if desired.is_some() || min_existing_version_id.is_some() {
            Some(Self {
                desired_state: desired.cloned(),
                legacy_tracking_table_names,
                source_state_table_always_exists: existing.always_exists()
                    && existing
                        .possible_versions()
                        .all(|v| v.source_state_table_name.is_some()),
                legacy_source_state_table_names,
                min_existing_version_id,
                source_names_need_state_cleanup,
                has_state_change: existing.has_state_diff(desired, |v| v),
            })
        } else {
            None
        }
    }

    pub fn into_setup_info(
        self,
    ) -> ResourceSetupInfo<(), TrackingTableSetupState, TrackingTableSetupChange> {
        ResourceSetupInfo {
            key: (),
            state: self.desired_state.clone(),
            has_tracked_state_change: self.has_state_change,
            description: "Internal Storage for Tracking".to_string(),
            setup_change: Some(self),
            legacy_key: None,
        }
    }
}

impl ResourceSetupChange for TrackingTableSetupChange {
    fn describe_changes(&self) -> Vec<setup::ChangeDescription> {
        let mut changes: Vec<setup::ChangeDescription> = vec![];
        if self.desired_state.is_some() && !self.legacy_tracking_table_names.is_empty() {
            changes.push(setup::ChangeDescription::Action(format!(
                "Rename legacy tracking tables: {}. ",
                self.legacy_tracking_table_names.iter().join(", ")
            )));
        }
        match (self.min_existing_version_id, &self.desired_state) {
            (None, Some(state)) => {
                changes.push(setup::ChangeDescription::Action(format!(
                    "Create the tracking table: {}. ",
                    state.table_name
                )));
            }
            (Some(min_version_id), Some(desired)) => {
                if min_version_id < desired.version_id {
                    changes.push(setup::ChangeDescription::Action(
                        "Update the tracking table. ".into(),
                    ));
                }
            }
            (Some(_), None) => changes.push(setup::ChangeDescription::Action(format!(
                "Drop existing tracking table: {}. ",
                self.legacy_tracking_table_names.iter().join(", ")
            ))),
            (None, None) => (),
        }

        let source_state_table_name = self
            .desired_state
            .as_ref()
            .and_then(|v| v.source_state_table_name.as_ref());
        if let Some(source_state_table_name) = source_state_table_name {
            if !self.legacy_source_state_table_names.is_empty() {
                changes.push(setup::ChangeDescription::Action(format!(
                    "Rename legacy source state tables: {}. ",
                    self.legacy_source_state_table_names.iter().join(", ")
                )));
            }
            if !self.source_state_table_always_exists {
                changes.push(setup::ChangeDescription::Action(format!(
                    "Create the source state table: {}. ",
                    source_state_table_name
                )));
            }
        } else if !self.source_state_table_always_exists
            && !self.legacy_source_state_table_names.is_empty()
        {
            changes.push(setup::ChangeDescription::Action(format!(
                "Drop existing source state table: {}. ",
                self.legacy_source_state_table_names.iter().join(", ")
            )));
        }

        if !self.source_names_need_state_cleanup.is_empty() {
            changes.push(setup::ChangeDescription::Action(format!(
                "Clean up legacy source states: {}. ",
                self.source_names_need_state_cleanup
                    .values()
                    .flatten()
                    .dedup()
                    .join(", ")
            )));
        }
        changes
    }

    fn change_type(&self) -> SetupChangeType {
        match (self.min_existing_version_id, &self.desired_state) {
            (None, Some(_)) => SetupChangeType::Create,
            (Some(min_version_id), Some(desired)) => {
                let source_state_table_up_to_date = self.legacy_source_state_table_names.is_empty()
                    && self.source_names_need_state_cleanup.is_empty()
                    && (self.source_state_table_always_exists
                        || desired.source_state_table_name.is_none());

                if min_version_id == desired.version_id
                    && self.legacy_tracking_table_names.is_empty()
                    && source_state_table_up_to_date
                {
                    SetupChangeType::NoChange
                } else if min_version_id < desired.version_id || !source_state_table_up_to_date {
                    SetupChangeType::Update
                } else {
                    SetupChangeType::Invalid
                }
            }
            (Some(_), None) => SetupChangeType::Delete,
            (None, None) => SetupChangeType::NoChange,
        }
    }
}
