use super::*;
use docnav_protocol::{
    AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoArguments, InfoResult,
    Manifest, Operation, OutlineArguments, OutlineResult, ProbeReason, ProbeReasonCode,
    ProbeResult, ProtocolDiagnosticCode, ReadArguments, ReadResult, RequestEnvelope,
    MANIFEST_VERSION, PROBE_VERSION,
};
use docnav_typed_fields::{FieldDef, ProcessingInputKind, ProcessingLocator};

// @case WB-CONTRACTS-ERROR-001
#[test]
fn adapter_error_constructors_project_protocol_error_details() {
    let not_found = AdapterError::document_not_found("missing.md").protocol_error();

    assert_eq!(not_found.code(), ProtocolDiagnosticCode::DocumentNotFound);
    assert_eq!(not_found.owner(), "adapter");
    assert_eq!(
        not_found.location(),
        Some(&serde_json::json!({ "path": "missing.md" }))
    );
    assert_eq!(
        not_found.guidance().unwrap()[0],
        "Check the document path and retry."
    );

    let issue = NativeOptionIssue {
        owner: "docnav-markdown".to_owned(),
        namespace: "options".to_owned(),
        key: "max_heading_level".to_owned(),
        source: "cli".to_owned(),
        reason_code: "above_maximum".to_owned(),
        field: "--max-heading-level".to_owned(),
        received: Some("7".to_owned()),
        expected: Some("integer in range 1..6".to_owned()),
        type_variant: Some("integer".to_owned()),
        config_source: None,
    };
    let invalid = AdapterError::native_option_invalid(
        "invalid max heading level",
        issue,
        ["Use --max-heading-level between 1 and 6.".to_owned()],
    )
    .protocol_error();

    assert_eq!(invalid.code(), ProtocolDiagnosticCode::InvalidRequest);
    assert_eq!(invalid.owner(), "adapter_options");
    assert_eq!(invalid.received(), Some(&serde_json::json!("7")));
    assert_eq!(
        invalid.expected(),
        Some(&serde_json::json!("integer in range 1..6"))
    );
    assert_eq!(
        invalid
            .details()
            .get("reason")
            .and_then(serde_json::Value::as_str),
        Some("above_maximum")
    );
    let option_issue = invalid
        .details()
        .get("option_issues")
        .and_then(serde_json::Value::as_array)
        .and_then(|issues| issues.first())
        .expect("option issue is projected");
    assert_eq!(option_issue["owner"], "docnav-markdown");
    assert_eq!(
        invalid.guidance().unwrap()[0],
        "Use --max-heading-level between 1 and 6."
    );
}

// @case WB-CONTRACTS-NATIVE-001
#[test]
fn adapter_options_keep_same_target_key_with_distinct_identity_and_owner() {
    let integer_variant = AdapterOptionSpec::builder("docnav.adapters.integer.options.shared")
        .owner("integer-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "integer-adapter", "shared"]),
        )
        .validation(FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(3)))
        .build();
    let string_variant = AdapterOptionSpec::builder("docnav.adapters.string.options.shared")
        .owner("string-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "string-adapter", "shared"]),
        )
        .validation(FieldValidation::string())
        .build();

    assert_eq!(integer_variant.key(), string_variant.key());
    assert_ne!(integer_variant.identity, string_variant.identity);
    assert_ne!(integer_variant.owner, string_variant.owner);
    assert_eq!(integer_variant.value_kind(), ValueKind::Integer);
    assert_eq!(string_variant.value_kind(), ValueKind::String);
    assert_eq!(integer_variant.namespace(), string_variant.namespace());
    assert_ne!(integer_variant.value_kind(), string_variant.value_kind());
}

#[test]
fn adapter_option_builder_wraps_typed_field_declaration_and_bindings() {
    let outline_only =
        AdapterOptionSpec::builder("docnav.adapters.markdown.options.max_heading_level")
            .owner("docnav-markdown")
            .operations(&[Operation::Outline])
            .path(["options", "max_heading_level"])
            .process(
                "cli",
                AdapterOptionProcessStrategy::cli_flag("--max-heading-level"),
            )
            .process(
                "config",
                AdapterOptionProcessStrategy::config_path([
                    "options",
                    "docnav-markdown",
                    "max_heading_level",
                ]),
            )
            .validation(
                FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
            )
            .default_static(3)
            .build();
    let read_only = AdapterOptionSpec::builder("docnav.adapters.markdown.options.read_mode")
        .owner("docnav-markdown")
        .operations(&[Operation::Read])
        .path(["options", "read_mode"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", "docnav-markdown", "read_mode"]),
        )
        .validation(FieldValidation::string())
        .build();

    let mut builder = FieldDefSet::builder().field(
        FieldDef::builder("docnav.defaults.pagination.limit")
            .process("direct", ProcessStrategy::json_path(["limit"]))
            .validation(FieldValidation::int()),
        ExpectedFieldShape::required(),
    );
    for option in [&outline_only, &read_only]
        .into_iter()
        .filter(|option| option.applies_to(Operation::Outline))
    {
        builder = builder.field_declaration(
            option
                .field_declaration()
                .expect("valid adapter option declaration"),
        );
    }
    let fields = builder.build().expect("adapter option field defs");

    let explicit = fields.processing_metadata(&ProcessingId::from("cli"));
    let config = fields.processing_metadata(&ProcessingId::from("config"));
    let direct = fields.processing_metadata(&ProcessingId::from("direct"));

    assert_eq!(direct.len(), 1);
    assert_eq!(direct[0].input_kind, ProcessingInputKind::JsonValue);
    assert_eq!(explicit.len(), 1);
    assert_eq!(explicit[0].input_kind, ProcessingInputKind::CliArguments);
    assert_eq!(
        explicit[0].identity.as_str(),
        "docnav.adapters.markdown.options.max_heading_level"
    );
    assert_eq!(
        explicit[0].locator,
        ProcessingLocator::CliFlag("--max-heading-level".to_owned())
    );
    assert_eq!(explicit[0].value_kind, ValueKind::Integer);
    assert_eq!(explicit[0].default, DefaultMetadata::Static(3.into()));
    assert_eq!(config.len(), 1);
    assert!(matches!(
        &config[0].locator,
        ProcessingLocator::ConfigPath(path)
            if path.segments()
                == vec!["options", "docnav-markdown", "max_heading_level"]
    ));
    assert_eq!(outline_only.cli_arg_id(), Some("max-heading-level"));
    assert_eq!(outline_only.processing_path("cli").unwrap(), None);
    assert_eq!(
        outline_only.cli_input_path().unwrap(),
        vec!["options", "max_heading_level"]
    );
    assert_eq!(
        outline_only.processing_path("config").unwrap().unwrap(),
        vec![
            "options".to_owned(),
            "docnav-markdown".to_owned(),
            "max_heading_level".to_owned(),
        ]
    );
    assert_eq!(
        outline_only.expected_value_description(),
        "integer in range 1..6"
    );
}

#[test]
fn adapter_option_field_declaration_rejects_invalid_path() {
    let invalid = AdapterOptionSpec::builder("docnav.adapters.markdown.options.max_heading_level")
        .owner("docnav-markdown")
        .operations(&[Operation::Outline])
        .path(["markdown", "max_heading_level"])
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path([
                "options",
                "docnav-markdown",
                "max_heading_level",
            ]),
        )
        .validation(FieldValidation::int())
        .build();

    let error = invalid
        .field_declaration()
        .expect_err("invalid declaration path");

    assert_eq!(
        error.to_string(),
        "adapter option docnav.adapters.markdown.options.max_heading_level declaration path must be options.<key>, got markdown.max_heading_level"
    );
}

// @case WB-CONTRACTS-DEFINITION-001
#[test]
fn adapter_definition_requires_all_operation_handlers() {
    let adapter = NoHookAdapter;
    let error = AdapterDefinition::builder("no-hook")
        .adapter(&adapter)
        .manifest(adapter.manifest())
        .operation_handler(Operation::Outline)
        .operation_handler(Operation::Read)
        .operation_handler(Operation::Find)
        .build()
        .expect_err("missing info handler");

    assert!(matches!(
        error,
        AdapterDefinitionError::MissingRequiredHandlers { operations, .. }
            if operations == vec![Operation::Info]
    ));
}

#[test]
fn adapter_definition_rejects_invalid_and_duplicate_native_options() {
    let adapter = NoHookAdapter;
    let invalid = AdapterOptionSpec::builder("docnav.adapters.no-hook.options.bad")
        .owner("no-hook")
        .operations(&[Operation::Outline])
        .path(["invalid", "bad"])
        .validation(FieldValidation::int())
        .build();
    let error = definition_builder(&adapter)
        .native_option(invalid)
        .build()
        .expect_err("invalid option path");
    assert!(matches!(
        error,
        AdapterDefinitionError::InvalidNativeOption { .. }
    ));

    let shared = no_hook_option("docnav.adapters.no-hook.options.shared", "shared");
    let error = definition_builder(&adapter)
        .native_option(shared.clone())
        .native_option(shared)
        .build()
        .expect_err("duplicate native option identity");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateNativeOptionDeclaration { .. }
    ));

    let error = definition_builder(&adapter)
        .native_option(no_hook_option(
            "docnav.adapters.no-hook.options.first",
            "shared",
        ))
        .native_option(no_hook_option(
            "docnav.adapters.no-hook.options.second",
            "shared",
        ))
        .build()
        .expect_err("duplicate native option path");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateNativeOptionPath { .. }
    ));
}

#[test]
fn adapter_definition_rejects_native_option_owner_mismatch() {
    let adapter = NoHookAdapter;
    let mismatched_owner = AdapterOptionSpec::builder("docnav.adapters.other.options.shared")
        .owner("other-adapter")
        .operations(&[Operation::Outline])
        .path(["options", "shared"])
        .validation(FieldValidation::int())
        .build();

    let error = definition_builder(&adapter)
        .native_option(mismatched_owner)
        .build()
        .expect_err("native option owner mismatch");

    assert!(matches!(
        error,
        AdapterDefinitionError::NativeOptionOwnerMismatch {
            id,
            owner,
            ..
        } if id == "no-hook" && owner == "other-adapter"
    ));
}

#[test]
fn adapter_definition_rejects_duplicate_handlers_and_capability_groups() {
    let adapter = NoHookAdapter;
    let error = definition_builder(&adapter)
        .operation_handler(Operation::Outline)
        .build()
        .expect_err("duplicate outline handler");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateOperationHandler {
            operation: Operation::Outline,
            ..
        }
    ));

    let capabilities = UnstructuredFullReadCapabilities {
        content_hook: true,
        cost_measurement_units: Vec::new(),
        result_facts_hook: false,
    };
    let error = definition_builder(&adapter)
        .full_read_capability_group(capabilities.clone())
        .full_read_capability_group(capabilities)
        .build()
        .expect_err("duplicate full-read group");
    assert!(matches!(
        error,
        AdapterDefinitionError::DuplicateCapabilityGroup {
            capability: "full_read",
            ..
        }
    ));
}

#[test]
fn adapter_definition_rejects_unsupported_full_read_capability_group() {
    let adapter = NoHookAdapter;
    let error = definition_builder(&adapter)
        .full_read_capability_group(UnstructuredFullReadCapabilities::default())
        .build()
        .expect_err("empty full-read group");

    assert!(matches!(
        error,
        AdapterDefinitionError::UnsupportedCapabilityCombination {
            capability: "full_read",
            ..
        }
    ));
}

#[test]
fn native_option_handoff_preserves_handler_facing_typed_metadata() {
    let mut options = docnav_protocol::Options::new();
    options.insert_entry(docnav_protocol::OptionEntry {
        identity: "docnav.adapters.no-hook.options.max".to_owned(),
        owner: "no-hook".to_owned(),
        namespace: "options".to_owned(),
        key: "max".to_owned(),
        source: "project".to_owned(),
        type_variant: "integer".to_owned(),
        value: serde_json::json!(4),
    });

    let handoff = NativeOptionHandoff::from_options(Some(&options));
    let value = handoff
        .get("no-hook", "options", "max")
        .expect("typed option value");

    assert_eq!(value.identity, "docnav.adapters.no-hook.options.max");
    assert_eq!(value.owner, "no-hook");
    assert_eq!(value.namespace, "options");
    assert_eq!(value.key, "max");
    assert_eq!(value.source, "project");
    assert_eq!(value.type_variant, "integer");
    assert_eq!(value.value, serde_json::json!(4));
}

// @case WB-CONTRACTS-UNSTRUCTURED-001
#[test]
fn unstructured_full_read_hooks_default_to_absent_capabilities() {
    let adapter = NoHookAdapter;
    let request = RequestEnvelope {
        protocol_version: docnav_protocol::PROTOCOL_VERSION.to_owned(),
        request_id: "req-hooks".to_owned(),
        operation: Operation::Outline,
        document: docnav_protocol::Document {
            path: "doc.stub".to_owned(),
        },
        arguments: docnav_protocol::OperationArguments::Outline(OutlineArguments {
            limit: docnav_protocol::positive_result(80).unwrap(),
            page: docnav_protocol::positive_result(1).unwrap(),
            options: None,
        }),
    };

    let capabilities = adapter.unstructured_full_read_capabilities();

    assert!(!capabilities.content_hook);
    assert!(!capabilities.result_facts_hook);
    assert!(capabilities.cost_measurement_units.is_empty());
    assert!(!capabilities.has_cost_measurement_unit("tokens"));
    assert!(adapter.unstructured_full_read(&request).is_err());
    assert_eq!(
        adapter
            .measure_unstructured_full_read_cost(&request, &["tokens".to_owned()])
            .unwrap()
            .measurements,
        Vec::new()
    );
    assert_eq!(
        adapter.unstructured_full_read_facts(&request).unwrap(),
        UnstructuredFullReadFacts::default()
    );
}

struct NoHookAdapter;

fn definition_builder(adapter: &NoHookAdapter) -> AdapterDefinitionBuilder<'_> {
    AdapterDefinition::builder("no-hook")
        .adapter(adapter)
        .manifest(adapter.manifest())
        .required_operation_handlers()
}

fn no_hook_option(identity: &str, key: &str) -> AdapterOptionSpec {
    AdapterOptionSpec::builder(identity)
        .owner("no-hook")
        .operations(&[Operation::Outline])
        .path(["options", key])
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
