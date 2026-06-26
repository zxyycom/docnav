use std::collections::BTreeMap;
use std::fmt;

use docnav_typed_fields::{
    FieldDefSet, FieldIdentity, ProcessingId, ProcessingInputKind, ProcessingMetadataView,
    SchemaMetadataView,
};

use crate::pipeline::StandardParameterPipelineSourceRole;
use crate::StandardParameterPath;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct OperationArgumentBinding {
    pub(crate) arguments_path: StandardParameterPath,
}

impl OperationArgumentBinding {
    #[cfg(test)]
    pub(crate) fn new(arguments_path: StandardParameterPath) -> Self {
        Self { arguments_path }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StandardParameterCatalogEntry {
    pub(crate) metadata: SchemaMetadataView,
    pub(crate) direct_input_path: Option<StandardParameterPath>,
    pub(crate) config_path: Option<StandardParameterPath>,
    pub(crate) operation_argument: Option<OperationArgumentBinding>,
}

impl StandardParameterCatalogEntry {
    pub(crate) fn new(metadata: SchemaMetadataView) -> Self {
        Self {
            metadata,
            direct_input_path: None,
            config_path: None,
            operation_argument: None,
        }
    }

    pub(crate) fn with_direct_input_path(mut self, path: StandardParameterPath) -> Self {
        self.direct_input_path = Some(path);
        self
    }

    pub(crate) fn with_config_path(mut self, path: StandardParameterPath) -> Self {
        self.config_path = Some(path);
        self
    }

    #[cfg(test)]
    pub(crate) fn with_operation_argument(mut self, binding: OperationArgumentBinding) -> Self {
        self.operation_argument = Some(binding);
        self
    }

    pub(crate) fn identity(&self) -> &FieldIdentity {
        &self.metadata.identity
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StandardParameterCatalog {
    entries: Vec<StandardParameterCatalogEntry>,
}

impl StandardParameterCatalog {
    pub(crate) fn new(entries: Vec<StandardParameterCatalogEntry>) -> Self {
        Self { entries }
    }

    pub(crate) fn entries(&self) -> &[StandardParameterCatalogEntry] {
        &self.entries
    }
}

pub(crate) fn derive_standard_parameter_catalog<D>(
    fields: &D,
    direct_processing: &ProcessingId,
    config_processing: &ProcessingId,
) -> Result<StandardParameterCatalog, StandardParameterCatalogError>
where
    D: AsRef<FieldDefSet> + ?Sized,
{
    let fields = fields.as_ref();
    let direct_paths = processing_paths(
        fields.processing_metadata(direct_processing),
        StandardParameterPipelineSourceRole::DirectInput,
    )?;
    let config_paths = processing_paths(
        fields.processing_metadata(config_processing),
        StandardParameterPipelineSourceRole::Config,
    )?;

    let entries = fields
        .schema_metadata()
        .into_iter()
        .map(|metadata| {
            let identity = metadata.identity.clone();
            let mut entry = StandardParameterCatalogEntry::new(metadata);
            if let Some(path) = direct_paths.get(&identity) {
                entry = entry.with_direct_input_path(path.clone());
            }
            if let Some(path) = config_paths.get(&identity) {
                entry = entry.with_config_path(path.clone());
            }
            entry
        })
        .collect::<Vec<_>>();

    validate_catalog_paths(&entries)?;
    Ok(StandardParameterCatalog::new(entries))
}

#[derive(Clone, Debug, PartialEq)]
pub enum StandardParameterCatalogError {
    ProcessingInputKindMismatch {
        role: StandardParameterPipelineSourceRole,
        processing_id: ProcessingId,
        identity: FieldIdentity,
        input_kind: ProcessingInputKind,
    },
    Conflict {
        kind: StandardParameterCatalogConflictKind,
        identity: FieldIdentity,
        previous_identity: FieldIdentity,
        path: Option<StandardParameterPath>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StandardParameterCatalogConflictKind {
    DirectInputPath,
    ConfigPath,
    OperationArgumentPath,
}

impl fmt::Display for StandardParameterCatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ProcessingInputKindMismatch {
                role,
                processing_id,
                identity,
                input_kind,
            } => write!(
                formatter,
                "{} processing {} for {} uses {:?}, but pipeline sources require JSON input",
                role.as_str(),
                processing_id,
                identity.as_str(),
                input_kind
            ),
            Self::Conflict {
                kind,
                identity,
                previous_identity,
                path: Some(path),
            } => write!(
                formatter,
                "{kind:?} catalog conflict for {} at {} (previously used by {})",
                identity.as_str(),
                path.segments().join("."),
                previous_identity.as_str()
            ),
            Self::Conflict {
                kind,
                identity,
                previous_identity,
                path: None,
            } => write!(
                formatter,
                "{kind:?} catalog conflict for {} (previously used by {})",
                identity.as_str(),
                previous_identity.as_str()
            ),
        }
    }
}

impl std::error::Error for StandardParameterCatalogError {}

fn processing_paths(
    metadata: Vec<ProcessingMetadataView>,
    role: StandardParameterPipelineSourceRole,
) -> Result<BTreeMap<FieldIdentity, StandardParameterPath>, StandardParameterCatalogError> {
    let mut paths = BTreeMap::new();
    for metadata in metadata {
        if metadata.input_kind != ProcessingInputKind::JsonValue {
            return Err(StandardParameterCatalogError::ProcessingInputKindMismatch {
                role,
                processing_id: metadata.processing_id,
                identity: metadata.identity,
                input_kind: metadata.input_kind,
            });
        }
        let path = StandardParameterPath::new(metadata.path.segments())
            .expect("typed-field processing path is validated before metadata projection");
        paths.insert(metadata.identity, path);
    }
    Ok(paths)
}

fn validate_catalog_paths(
    entries: &[StandardParameterCatalogEntry],
) -> Result<(), StandardParameterCatalogError> {
    let mut direct_paths = BTreeMap::new();
    let mut config_paths = BTreeMap::new();
    let mut operation_paths = BTreeMap::new();

    for entry in entries {
        check_path_conflict(
            &mut direct_paths,
            StandardParameterCatalogConflictKind::DirectInputPath,
            entry.identity(),
            entry.direct_input_path.as_ref(),
        )?;
        check_path_conflict(
            &mut config_paths,
            StandardParameterCatalogConflictKind::ConfigPath,
            entry.identity(),
            entry.config_path.as_ref(),
        )?;
        check_path_conflict(
            &mut operation_paths,
            StandardParameterCatalogConflictKind::OperationArgumentPath,
            entry.identity(),
            entry
                .operation_argument
                .as_ref()
                .map(|binding| &binding.arguments_path),
        )?;
    }

    Ok(())
}

fn check_path_conflict(
    paths: &mut BTreeMap<Vec<String>, FieldIdentity>,
    kind: StandardParameterCatalogConflictKind,
    identity: &FieldIdentity,
    path: Option<&StandardParameterPath>,
) -> Result<(), StandardParameterCatalogError> {
    let Some(path) = path else {
        return Ok(());
    };
    if let Some(previous) = paths.insert(path.key(), identity.clone()) {
        return Err(StandardParameterCatalogError::Conflict {
            kind,
            identity: identity.clone(),
            previous_identity: previous,
            path: Some(path.clone()),
        });
    }
    Ok(())
}
