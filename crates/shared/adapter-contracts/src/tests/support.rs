use crate::{
    Adapter, AdapterDefinition, AdapterDefinitionBuilder, AdapterOptionProcessStrategy,
    AdapterOptionSpec, AdapterResult, FieldValidation,
};
use docnav_protocol::{
    AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoArguments, InfoResult,
    Manifest, Operation, OutlineArguments, OutlineResult, ProbeReason, ProbeReasonCode,
    ProbeResult, ReadArguments, ReadResult, RequestEnvelope, MANIFEST_VERSION, PROBE_VERSION,
};

pub(super) struct NoHookAdapter;

pub(super) fn definition_builder(adapter: &NoHookAdapter) -> AdapterDefinitionBuilder<'_> {
    AdapterDefinition::builder("no-hook")
        .adapter(adapter)
        .manifest(adapter.manifest())
        .required_operation_handlers()
}

pub(super) fn no_hook_option(identity: &str, key: &str) -> AdapterOptionSpec {
    AdapterOptionSpec::builder(identity)
        .owner("no-hook")
        .operations(&[Operation::Outline])
        .path(["options", key])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "no-hook", key]),
        )
        .validation(FieldValidation::int())
        .build()
}

impl Adapter for NoHookAdapter {
    fn adapter_id(&self) -> &str {
        "no-hook"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: MANIFEST_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "no-hook".to_owned(),
                name: "No Hook".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "stub".to_owned(),
                extensions: vec![".stub".to_owned()],
                content_types: vec!["text/stub".to_owned()],
            }],
        }
    }

    fn probe(&self, path: &str) -> ProbeResult {
        ProbeResult {
            probe_version: PROBE_VERSION.to_owned(),
            adapter_id: self.adapter_id().to_owned(),
            path: path.to_owned(),
            supported: true,
            format: Some("stub".to_owned()),
            confidence: 1.0,
            reasons: vec![ProbeReason {
                code: ProbeReasonCode::ContentMatch,
                detail: "test adapter".to_owned(),
            }],
        }
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        unreachable!("unstructured hook test does not dispatch outline")
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        unreachable!("unstructured hook test does not dispatch read")
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        unreachable!("unstructured hook test does not dispatch find")
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        unreachable!("unstructured hook test does not dispatch info")
    }
}
