use crate::{
    ops::{
        get_attachment_factory, get_optional_target_factory,
        interface::{AttachmentSetupKey, FlowInstanceContext, TargetFactory},
    },
    prelude::*,
    setup::{AttachmentsSetupChange, TargetSetupChange},
};

use sqlx::PgPool;
use std::{
    fmt::{Debug, Display},
    str::FromStr,
};

use super::AllSetupStates;
use super::{
    CombinedState, DesiredMode, ExistingMode, FlowSetupChange, FlowSetupState, ObjectStatus,
    ResourceIdentifier, ResourceSetupInfo, StateChange, TargetSetupState, db_metadata,
};
use crate::execution::db_tracking_setup;

enum MetadataRecordType {
    FlowVersion,
    FlowMetadata,
    TrackingTable,
    Target(String),
}

impl Display for MetadataRecordType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MetadataRecordType::FlowVersion => f.write_str(db_metadata::FLOW_VERSION_RESOURCE_TYPE),
            MetadataRecordType::FlowMetadata => write!(f, "FlowMetadata"),
            MetadataRecordType::TrackingTable => write!(f, "TrackingTable"),
            MetadataRecordType::Target(target_id) => write!(f, "Target:{target_id}"),
        }
    }
}

impl std::str::FromStr for MetadataRecordType {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s == db_metadata::FLOW_VERSION_RESOURCE_TYPE {
            Ok(Self::FlowVersion)
        } else if s == "FlowMetadata" {
            Ok(Self::FlowMetadata)
        } else if s == "TrackingTable" {
            Ok(Self::TrackingTable)
        } else if let Some(target_id) = s.strip_prefix("Target:") {
            Ok(Self::Target(target_id.to_string()))
        } else {
            internal_bail!("Invalid MetadataRecordType string: {}", s)
        }
    }
}

fn from_metadata_record<S: DeserializeOwned + Debug + Clone>(
    state: Option<serde_json::Value>,
    staging_changes: sqlx::types::Json<Vec<StateChange<serde_json::Value>>>,
    legacy_state_key: Option<serde_json::Value>,
) -> Result<CombinedState<S>> {
    let current: Option<S> = state.map(utils::deser::from_json_value).transpose()?;
    let staging: Vec<StateChange<S>> = (staging_changes.0.into_iter())
        .map(|sc| -> Result<_> {
            Ok(match sc {
                StateChange::Upsert(v) => StateChange::Upsert(utils::deser::from_json_value(v)?),
                StateChange::Delete => StateChange::Delete,
            })
        })
        .collect::<Result<_>>()?;
    Ok(CombinedState {
        current,
        staging,
        legacy_state_key,
    })
}

fn get_export_target_factory(target_type: &str) -> Option<Arc<dyn TargetFactory + Send + Sync>> {
    get_optional_target_factory(target_type)
}

pub async fn get_existing_setup_state(pool: &PgPool) -> Result<AllSetupStates<ExistingMode>> {
    let setup_metadata_records = db_metadata::read_setup_metadata(pool).await?;

    let setup_metadata_records = if let Some(records) = setup_metadata_records {
        records
    } else {
        return Ok(AllSetupStates::default());
    };

    // Group setup metadata records by flow name
    let setup_metadata_records = setup_metadata_records.into_iter().fold(
        BTreeMap::<String, Vec<_>>::new(),
        |mut acc, record| {
            acc.entry(record.flow_name.clone())
                .or_default()
                .push(record);
            acc
        },
    );

    let flows = setup_metadata_records
        .into_iter()
        .map(|(flow_name, metadata_records)| -> Result<_> {
            let mut flow_ss = FlowSetupState::default();
            for metadata_record in metadata_records {
                let state = metadata_record.state;
                let staging_changes = metadata_record.staging_changes;
                match MetadataRecordType::from_str(&metadata_record.resource_type)? {
                    MetadataRecordType::FlowVersion => {
                        flow_ss.seen_flow_metadata_version =
                            db_metadata::parse_flow_version(&state);
                    }
                    MetadataRecordType::FlowMetadata => {
                        flow_ss.metadata = from_metadata_record(state, staging_changes, None)?;
                    }
                    MetadataRecordType::TrackingTable => {
                        flow_ss.tracking_table =
                            from_metadata_record(state, staging_changes, None)?;
                    }
                    MetadataRecordType::Target(target_type) => {
                        let normalized_key = {
                            if let Some(factory) = get_export_target_factory(&target_type) {
                                factory.normalize_setup_key(&metadata_record.key)?
                            } else {
                                metadata_record.key.clone()
                            }
                        };
                        let combined_state = from_metadata_record(
                            state,
                            staging_changes,
                            (normalized_key != metadata_record.key).then_some(metadata_record.key),
                        )?;
                        flow_ss.targets.insert(
                            super::ResourceIdentifier {
                                key: normalized_key,
                                target_kind: target_type,
                            },
                            combined_state,
                        );
                    }
                }
            }
            Ok((flow_name, flow_ss))
        })
        .collect::<Result<_>>()?;

    Ok(AllSetupStates { flows })
}

fn diff_state<E, D, T>(
    existing_state: Option<&E>,
    desired_state: Option<&D>,
    diff: impl Fn(Option<&E>, &D) -> Option<StateChange<T>>,
) -> Option<StateChange<T>>
where
    E: PartialEq<D>,
{
    match (existing_state, desired_state) {
        (None, None) => None,
        (Some(_), None) => Some(StateChange::Delete),
        (existing_state, Some(desired_state)) => {
            if existing_state.map(|e| e == desired_state).unwrap_or(false) {
                None
            } else {
                diff(existing_state, desired_state)
            }
        }
    }
}

fn to_object_status<A, B>(existing: Option<A>, desired: Option<B>) -> Option<ObjectStatus> {
    Some(match (&existing, &desired) {
        (Some(_), None) => ObjectStatus::Deleted,
        (None, Some(_)) => ObjectStatus::New,
        (Some(_), Some(_)) => ObjectStatus::Existing,
        (None, None) => return None,
    })
}

#[derive(Debug)]
struct GroupedResourceStates<S: Debug + Clone> {
    desired: Option<S>,
    existing: CombinedState<S>,
}

impl<S: Debug + Clone> Default for GroupedResourceStates<S> {
    fn default() -> Self {
        Self {
            desired: None,
            existing: CombinedState::default(),
        }
    }
}

fn group_states<K: Hash + Eq + std::fmt::Display + std::fmt::Debug + Clone, S: Debug + Clone>(
    desired: impl Iterator<Item = (K, S)>,
    existing: impl Iterator<Item = (K, CombinedState<S>)>,
) -> Result<IndexMap<K, GroupedResourceStates<S>>> {
    let mut grouped: IndexMap<K, GroupedResourceStates<S>> = desired
        .into_iter()
        .map(|(key, state)| {
            (
                key,
                GroupedResourceStates {
                    desired: Some(state.clone()),
                    existing: CombinedState::default(),
                },
            )
        })
        .collect();
    for (key, state) in existing {
        let entry = grouped.entry(key.clone());
        if state.current.is_some()
            && let indexmap::map::Entry::Occupied(entry) = &entry
            && entry.get().existing.current.is_some()
        {
            internal_bail!("Duplicate existing state for key: {}", entry.key());
        }
        let entry = entry.or_default();
        if let Some(current) = &state.current {
            entry.existing.current = Some(current.clone());
        }
        if let Some(legacy_state_key) = &state.legacy_state_key {
            if entry
                .existing
                .legacy_state_key
                .as_ref()
                .is_some_and(|v| v != legacy_state_key)
            {
                warn!(
                    "inconsistent legacy key: {key}, {:?}",
                    entry.existing.legacy_state_key
                );
            }
            entry.existing.legacy_state_key = Some(legacy_state_key.clone());
        }
        for s in state.staging.iter() {
            match s {
                StateChange::Upsert(v) => {
                    entry.existing.staging.push(StateChange::Upsert(v.clone()))
                }
                StateChange::Delete => entry.existing.staging.push(StateChange::Delete),
            }
        }
    }
    Ok(grouped)
}

async fn collect_attachments_setup_change(
    target_key: &serde_json::Value,
    desired: Option<&TargetSetupState>,
    existing: &CombinedState<TargetSetupState>,
    context: &interface::FlowInstanceContext,
) -> Result<AttachmentsSetupChange> {
    let existing_current_attachments = existing
        .current
        .iter()
        .flat_map(|s| s.attachments.iter())
        .map(|(key, state)| (key.clone(), CombinedState::current(state.clone())));
    let existing_staging_attachments = existing.staging.iter().flat_map(|s| {
        match s {
            StateChange::Upsert(s) => Some(s.attachments.iter().map(|(key, state)| {
                (
                    key.clone(),
                    CombinedState::staging(StateChange::Upsert(state.clone())),
                )
            })),
            StateChange::Delete => None,
        }
        .into_iter()
        .flatten()
    });
    let mut grouped_attachment_states = group_states(
        desired.iter().flat_map(|s| {
            s.attachments
                .iter()
                .map(|(key, state)| (key.clone(), state.clone()))
        }),
        (existing_current_attachments.into_iter())
            .chain(existing_staging_attachments)
            .rev(),
    )?;
    if existing
        .staging
        .iter()
        .any(|s| matches!(s, StateChange::Delete))
    {
        for state in grouped_attachment_states.values_mut() {
            if state
                .existing
                .staging
                .iter()
                .all(|s| matches!(s, StateChange::Delete))
            {
                state.existing.staging.push(StateChange::Delete);
            }
        }
    }

    let mut attachments_change = AttachmentsSetupChange::default();
    for (AttachmentSetupKey(kind, key), setup_state) in grouped_attachment_states.into_iter() {
        let has_diff = setup_state
            .existing
            .has_state_diff(setup_state.desired.as_ref(), |s| s);
        if !has_diff {
            continue;
        }
        attachments_change.has_tracked_state_change = true;
        let factory = get_attachment_factory(&kind)?;
        let is_upsertion = setup_state.desired.is_some();
        if let Some(action) = factory
            .diff_setup_states(
                target_key,
                &key,
                setup_state.desired,
                setup_state.existing,
                context,
            )
            .await?
        {
            if is_upsertion {
                attachments_change.upserts.push(action);
            } else {
                attachments_change.deletes.push(action);
            }
        }
    }
    Ok(attachments_change)
}

pub async fn diff_flow_setup_states(
    desired_state: Option<&FlowSetupState<DesiredMode>>,
    existing_state: Option<&FlowSetupState<ExistingMode>>,
    flow_instance_ctx: &Arc<FlowInstanceContext>,
) -> Result<FlowSetupChange> {
    let metadata_change = diff_state(
        existing_state.map(|e| &e.metadata),
        desired_state.map(|d| &d.metadata),
        |_, desired_state| Some(StateChange::Upsert(desired_state.clone())),
    );

    // If the source kind has changed, we need to clean the source states.
    let source_names_needs_states_cleanup: BTreeMap<i32, BTreeSet<String>> =
        if let Some(desired_state) = desired_state
            && let Some(existing_state) = existing_state
        {
            let new_source_id_to_kind = desired_state
                .metadata
                .sources
                .values()
                .map(|v| (v.source_id, &v.source_kind))
                .collect::<HashMap<i32, &String>>();

            let mut existing_source_id_to_name_kind =
                BTreeMap::<i32, Vec<(&String, &String)>>::new();
            for (name, setup_state) in existing_state
                .metadata
                .possible_versions()
                .flat_map(|v| v.sources.iter())
            {
                // For backward compatibility, we only process source states for non-empty source kinds.
                if !setup_state.source_kind.is_empty() {
                    existing_source_id_to_name_kind
                        .entry(setup_state.source_id)
                        .or_default()
                        .push((name, &setup_state.source_kind));
                }
            }

            (existing_source_id_to_name_kind.into_iter())
                .map(|(id, name_kinds)| {
                    let new_kind = new_source_id_to_kind.get(&id).copied();
                    let source_names_for_legacy_states = name_kinds
                        .into_iter()
                        .filter_map(|(name, kind)| {
                            if Some(kind) != new_kind {
                                Some(name.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<BTreeSet<_>>();
                    (id, source_names_for_legacy_states)
                })
                .filter(|(_, v)| !v.is_empty())
                .collect::<BTreeMap<_, _>>()
        } else {
            BTreeMap::new()
        };

    let tracking_table_change = db_tracking_setup::TrackingTableSetupChange::new(
        desired_state.map(|d| &d.tracking_table),
        &existing_state
            .map(|e| Cow::Borrowed(&e.tracking_table))
            .unwrap_or_default(),
        source_names_needs_states_cleanup,
    );

    let mut target_resources = Vec::new();
    let mut unknown_resources = Vec::new();

    let grouped_target_resources = group_states(
        desired_state
            .iter()
            .flat_map(|d| d.targets.iter().map(|(k, v)| (k.clone(), v.clone()))),
        existing_state
            .iter()
            .flat_map(|e| e.targets.iter().map(|(k, v)| (k.clone(), v.clone()))),
    )?;
    for (resource_id, target_states_group) in grouped_target_resources.into_iter() {
        let factory = match get_export_target_factory(&resource_id.target_kind) {
            Some(factory) => factory,
            None => {
                unknown_resources.push(resource_id.clone());
                continue;
            }
        };

        let attachments_change = collect_attachments_setup_change(
            &resource_id.key,
            target_states_group.desired.as_ref(),
            &target_states_group.existing,
            flow_instance_ctx,
        )
        .await?;

        let desired_state = target_states_group.desired.clone();
        let has_tracked_state_change = target_states_group
            .existing
            .has_state_diff(desired_state.as_ref().map(|s| &s.state), |s| &s.state)
            || attachments_change.has_tracked_state_change;
        let existing_without_setup_by_user = CombinedState {
            current: target_states_group
                .existing
                .current
                .and_then(|s| s.state_unless_setup_by_user()),
            staging: target_states_group
                .existing
                .staging
                .into_iter()
                .filter_map(|s| match s {
                    StateChange::Upsert(s) => {
                        s.state_unless_setup_by_user().map(StateChange::Upsert)
                    }
                    StateChange::Delete => Some(StateChange::Delete),
                })
                .collect(),
            legacy_state_key: target_states_group.existing.legacy_state_key.clone(),
        };
        let target_state_to_setup = target_states_group
            .desired
            .and_then(|state| (!state.common.setup_by_user).then_some(state.state));
        let never_setup_by_sys = target_state_to_setup.is_none()
            && existing_without_setup_by_user.current.is_none()
            && existing_without_setup_by_user.staging.is_empty();
        let setup_change = if never_setup_by_sys {
            None
        } else {
            Some(TargetSetupChange {
                target_change: factory
                    .diff_setup_states(
                        &resource_id.key,
                        target_state_to_setup,
                        existing_without_setup_by_user,
                        flow_instance_ctx.clone(),
                    )
                    .await?,
                attachments_change,
            })
        };

        target_resources.push(ResourceSetupInfo {
            key: resource_id.clone(),
            state: desired_state,
            has_tracked_state_change,
            description: factory.describe_resource(&resource_id.key)?,
            setup_change,
            legacy_key: target_states_group
                .existing
                .legacy_state_key
                .map(|legacy_state_key| ResourceIdentifier {
                    target_kind: resource_id.target_kind.clone(),
                    key: legacy_state_key,
                }),
        });
    }
    Ok(FlowSetupChange {
        status: to_object_status(existing_state, desired_state),
        seen_flow_metadata_version: existing_state.and_then(|s| s.seen_flow_metadata_version),
        metadata_change,
        tracking_table: tracking_table_change.map(|c| c.into_setup_info()),
        target_resources,
        unknown_resources,
    })
}
