use docnav_protocol::{validate_manifest_value, Manifest, ProbeResult};
use std::fmt;

use crate::adapter::Adapter;
use crate::constants::diagnostics;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AdapterBoundaryError {
    ManifestSchema(String),
    ManifestSemantic(String),
    ManifestAdapterIdMismatch {
        adapter_id: String,
        manifest_adapter_id: String,
    },
    ProbeSemantic(String),
    ProbeAdapterIdMismatch {
        adapter_id: String,
        manifest_adapter_id: String,
        probe_adapter_id: String,
    },
}

impl AdapterBoundaryError {
    pub(crate) const fn diagnostic(&self) -> &'static str {
        match self {
            Self::ManifestSemantic(_) => diagnostics::MANIFEST_SEMANTIC_VALIDATION_FAILED,
            Self::ManifestSchema(_) => diagnostics::MANIFEST_SCHEMA_VALIDATION_FAILED,
            Self::ManifestAdapterIdMismatch { .. } => diagnostics::MANIFEST_ADAPTER_ID_MISMATCH,
            Self::ProbeSemantic(_) => diagnostics::PROBE_RESULT_SEMANTIC_VALIDATION_FAILED,
            Self::ProbeAdapterIdMismatch { .. } => diagnostics::PROBE_RESULT_ADAPTER_ID_MISMATCH,
        }
    }
}

impl fmt::Display for AdapterBoundaryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ManifestSchema(error) | Self::ManifestSemantic(error) | Self::ProbeSemantic(error) => {
                formatter.write_str(error)
            }
            Self::ManifestAdapterIdMismatch {
                adapter_id,
                manifest_adapter_id,
            } => write!(
                formatter,
                "adapter_id() {adapter_id:?} must match manifest.adapter.id {manifest_adapter_id:?}"
            ),
            Self::ProbeAdapterIdMismatch {
                adapter_id,
                manifest_adapter_id,
                probe_adapter_id,
            } => write!(
                formatter,
                "adapter_id() {adapter_id:?}, manifest.adapter.id {manifest_adapter_id:?}, and probe.adapter_id {probe_adapter_id:?} must match"
            ),
        }
    }
}

pub(crate) fn validated_manifest<A: Adapter + ?Sized>(
    adapter: &A,
) -> Result<Manifest, AdapterBoundaryError> {
    let manifest = adapter.manifest();
    let manifest_value = serde_json::to_value(&manifest)
        .map_err(|error| AdapterBoundaryError::ManifestSchema(error.to_string()))?;
    validate_manifest_value(&manifest_value)
        .map_err(|error| AdapterBoundaryError::ManifestSchema(error.to_string()))?;
    manifest
        .validate_semantics()
        .map_err(|error| AdapterBoundaryError::ManifestSemantic(error.to_string()))?;

    let adapter_id = adapter.adapter_id();
    if manifest.adapter.id != adapter_id {
        return Err(AdapterBoundaryError::ManifestAdapterIdMismatch {
            adapter_id: adapter_id.to_owned(),
            manifest_adapter_id: manifest.adapter.id.clone(),
        });
    }

    Ok(manifest)
}

pub(crate) fn validated_probe<A: Adapter + ?Sized>(
    adapter: &A,
    manifest: &Manifest,
    path: &str,
) -> Result<ProbeResult, AdapterBoundaryError> {
    let probe = adapter.probe(path);
    probe
        .validate_semantics()
        .map_err(|error| AdapterBoundaryError::ProbeSemantic(error.to_string()))?;

    let adapter_id = adapter.adapter_id();
    if probe.adapter_id != adapter_id || probe.adapter_id != manifest.adapter.id {
        return Err(AdapterBoundaryError::ProbeAdapterIdMismatch {
            adapter_id: adapter_id.to_owned(),
            manifest_adapter_id: manifest.adapter.id.clone(),
            probe_adapter_id: probe.adapter_id.clone(),
        });
    }

    Ok(probe)
}
