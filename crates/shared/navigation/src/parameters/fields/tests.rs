use docnav_adapter_contracts::{
    Adapter, AdapterDefinition, AdapterOptionProcessStrategy, AdapterOptionSpec, AdapterResult,
    CliProcessingMetadata, FieldBound, FieldValidation,
};
use docnav_protocol::{
    AdapterIdentity, FindArguments, FindResult, FormatDescriptor, InfoArguments, InfoResult,
    Manifest, Operation, OutlineArguments, OutlineResult, ProbeResult, ReadArguments, ReadResult,
    RequestEnvelope,
};
use docnav_typed_fields::{
    FieldDefSet, FieldDefSetBuildError, FieldLength, ProcessingId, ProcessingLocator,
};

use crate::{
    document_cli_field_set, DocumentCliFieldOwner, DocumentCliFieldSetBuildError,
    NavigationAdapterRef, NavigationAdapterRegistry,
};

use super::{operation_fields, registry_cli_fields};

#[test]
fn document_cli_field_set_preserves_common_registry_and_declaration_order_per_operation() {
    let first_adapter = TestAdapter("first");
    let second_adapter = TestAdapter("second");
    let registry = ordered_registry(&first_adapter, &second_adapter);

    let outline = document_cli_field_set(Operation::Outline, &registry).expect("outline fields");
    let read = document_cli_field_set(Operation::Read, &registry)
        .expect("read fields")
        .into_fields();

    assert_eq!(
        cli_identities(outline.fields()),
        vec![
            "docnav.defaults.adapter",
            "docnav.defaults.output",
            "docnav.defaults.pagination.enabled",
            "docnav.document.page",
            "docnav.defaults.pagination.limit",
            "docnav.adapters.first.options.outline",
            "docnav.adapters.second.options.outline",
        ]
    );
    assert_eq!(
        cli_identities(&read),
        vec![
            "docnav.defaults.adapter",
            "docnav.defaults.output",
            "docnav.defaults.pagination.enabled",
            "docnav.document.page",
            "docnav.defaults.pagination.limit",
            "docnav.adapters.first.options.read",
            "docnav.adapters.second.options.read",
        ]
    );

    let native_identity = outline
        .fields()
        .schema_metadata()
        .into_iter()
        .find(|metadata| metadata.identity.as_str().ends_with(".options.outline"))
        .expect("outline native field")
        .identity;
    let attribution = outline
        .attribution(&native_identity)
        .expect("native field attribution");
    assert_eq!(attribution.owner(), DocumentCliFieldOwner::Adapter);
    assert_eq!(attribution.identity(), &native_identity);
    assert_eq!(
        attribution.declaration_path(),
        Some(["options".to_owned(), "outline".to_owned()].as_slice())
    );
    assert_eq!(attribution.adapter_id(), Some("first"));
}

// @case WB-NAVIGATION-FIELD-SETS-001
#[test]
fn selected_fields_reuse_registry_declarations_for_each_current_operation() {
    let first_adapter = TestAdapter("first");
    let second_adapter = TestAdapter("second");
    let registry = ordered_registry(&first_adapter, &second_adapter);

    for operation in [
        Operation::Outline,
        Operation::Read,
        Operation::Find,
        Operation::Info,
    ] {
        let registry_projection =
            registry_cli_fields(operation, &registry).expect("registry CLI fields");
        let registry_fields = registry_projection.fields();
        let selected_fields = operation_fields(operation, &registry.definitions[0])
            .expect("selected operation fields");

        let selected_native = format!("docnav.adapters.first.options.{}", operation_key(operation));
        let other_adapter = format!(
            "docnav.adapters.second.options.{}",
            operation_key(operation)
        );
        assert_has_identity(registry_fields, &selected_native);
        assert_has_identity(registry_fields, &other_adapter);
        assert_has_identity(selected_fields.as_ref(), &selected_native);
        assert_lacks_identity(selected_fields.as_ref(), &other_adapter);

        for inapplicable in [
            Operation::Outline,
            Operation::Read,
            Operation::Find,
            Operation::Info,
        ]
        .into_iter()
        .filter(|candidate| *candidate != operation)
        {
            let identity = format!(
                "docnav.adapters.first.options.{}",
                operation_key(inapplicable)
            );
            assert_lacks_identity(registry_fields, &identity);
            assert_lacks_identity(selected_fields.as_ref(), &identity);
        }

        assert_registry_selected_parity(registry_fields, selected_fields.as_ref());
    }
}

#[test]
fn document_cli_field_set_preserves_duplicate_locator_attribution() {
    let first_adapter = TestAdapter("first");
    let second_adapter = TestAdapter("second");
    let registry = TestRegistry {
        definitions: vec![
            definition(
                &first_adapter,
                vec![option(
                    "first",
                    "first-option",
                    "--shared",
                    Operation::Outline,
                )],
            ),
            definition(
                &second_adapter,
                vec![option(
                    "second",
                    "second-option",
                    "--shared",
                    Operation::Outline,
                )],
            ),
        ],
    };

    let error = document_cli_field_set(Operation::Outline, &registry)
        .expect_err("same-operation adapter flags must be unique");
    let message = error.to_string();
    let DocumentCliFieldSetBuildError::DuplicateField {
        previous,
        current,
        source,
    } = error
    else {
        panic!("expected attributed duplicate field");
    };
    let FieldDefSetBuildError::DuplicateProcessingLocator(error) = *source else {
        panic!("expected duplicate processing locator");
    };

    assert_eq!(previous.owner(), DocumentCliFieldOwner::Adapter);
    assert_eq!(previous.adapter_id(), Some("first"));
    assert_eq!(current.owner(), DocumentCliFieldOwner::Adapter);
    assert_eq!(current.adapter_id(), Some("second"));

    assert_eq!(error.processing_id.as_str(), "cli");
    assert_eq!(
        error.locator,
        ProcessingLocator::CliFlag("--shared".to_owned())
    );
    assert_eq!(
        error.previous_identity.as_str(),
        "docnav.adapters.first.options.first-option"
    );
    assert_eq!(
        error.previous_declaration_path,
        Some(vec!["options".to_owned(), "first-option".to_owned()])
    );
    assert_eq!(
        error.current_identity.as_str(),
        "docnav.adapters.second.options.second-option"
    );
    assert_eq!(
        error.current_declaration_path,
        Some(vec!["options".to_owned(), "second-option".to_owned()])
    );
    for attribution in [
        "docnav.adapters.first.options.first-option",
        "options.first-option",
        "docnav.adapters.second.options.second-option",
        "options.second-option",
    ] {
        assert!(
            message.contains(attribution),
            "missing {attribution}: {message}"
        );
    }
}

#[test]
fn registry_cli_fields_attribute_duplicate_locator_between_common_and_native_fields() {
    let adapter = TestAdapter("first");
    let registry = TestRegistry {
        definitions: vec![definition(
            &adapter,
            vec![option(
                "first",
                "adapter-alias",
                "--adapter",
                Operation::Outline,
            )],
        )],
    };

    let error = registry_cli_fields(Operation::Outline, &registry)
        .expect_err("native flags must not conflict with common flags");
    let message = error.to_string();
    let DocumentCliFieldSetBuildError::DuplicateField {
        previous,
        current,
        source,
    } = error
    else {
        panic!("expected attributed duplicate field");
    };
    let FieldDefSetBuildError::DuplicateProcessingLocator(error) = *source else {
        panic!("expected duplicate processing locator");
    };

    assert_eq!(previous.owner(), DocumentCliFieldOwner::Navigation);
    assert_eq!(previous.adapter_id(), None);
    assert_eq!(current.owner(), DocumentCliFieldOwner::Adapter);
    assert_eq!(current.adapter_id(), Some("first"));

    assert_eq!(error.processing_id.as_str(), "cli");
    assert_eq!(
        error.locator,
        ProcessingLocator::CliFlag("--adapter".to_owned())
    );
    assert_eq!(error.previous_identity.as_str(), "docnav.defaults.adapter");
    assert_eq!(
        error.previous_declaration_path,
        Some(vec!["adapter".to_owned()])
    );
    assert_eq!(
        error.current_identity.as_str(),
        "docnav.adapters.first.options.adapter-alias"
    );
    assert_eq!(
        error.current_declaration_path,
        Some(vec!["options".to_owned(), "adapter-alias".to_owned()])
    );
    for attribution in [
        "docnav.defaults.adapter",
        " at adapter",
        "docnav.adapters.first.options.adapter-alias",
        "options.adapter-alias",
    ] {
        assert!(
            message.contains(attribution),
            "missing {attribution}: {message}"
        );
    }
}

#[test]
fn registry_and_selected_fields_reject_malformed_adapter_config_locator() {
    let adapter = TestAdapter("first");
    let registry = TestRegistry {
        definitions: vec![definition(
            &adapter,
            vec![option_with_config_owner(
                "first",
                "outline",
                "--outline",
                Operation::Outline,
                "wrong-owner",
            )],
        )],
    };

    let registry_error = document_cli_field_set(Operation::Outline, &registry)
        .expect_err("registry projection must validate config locator");
    let DocumentCliFieldSetBuildError::InvalidAdapterOption {
        attribution,
        reason,
    } = registry_error
    else {
        panic!("expected attributed adapter option failure");
    };
    assert_eq!(attribution.owner(), DocumentCliFieldOwner::Adapter);
    assert_eq!(attribution.adapter_id(), Some("first"));
    assert_eq!(
        attribution.identity().as_str(),
        "docnav.adapters.first.options.outline"
    );
    assert_eq!(
        attribution.declaration_path(),
        Some(["options".to_owned(), "outline".to_owned()].as_slice())
    );
    assert!(reason.contains("options.first.outline"));
    assert!(reason.contains("options.wrong-owner.outline"));

    assert!(
        operation_fields(Operation::Outline, &registry.definitions[0]).is_err(),
        "selected projection must validate config locator"
    );
}

fn cli_identities(fields: &docnav_typed_fields::FieldDefSet) -> Vec<String> {
    fields
        .processing_metadata(&ProcessingId::new("cli").expect("valid CLI processing id"))
        .into_iter()
        .map(|metadata| metadata.identity.as_str().to_owned())
        .collect()
}

fn assert_registry_selected_parity(registry: &FieldDefSet, selected: &FieldDefSet) {
    let selected_schema = selected.schema_metadata();
    for selected_metadata in selected_schema {
        let identity = selected_metadata.identity.clone();
        let registry_metadata = registry
            .schema_metadata()
            .into_iter()
            .find(|metadata| metadata.identity == identity)
            .expect("independently asserted selected identity must exist in registry");
        assert_eq!(registry_metadata, selected_metadata);

        for processing in ["direct", "config", "cli"] {
            let processing_id = ProcessingId::new(processing).expect("valid processing id");
            let registry_metadata = registry
                .processing_metadata(&processing_id)
                .into_iter()
                .find(|metadata| metadata.identity == identity);
            let selected_metadata = selected
                .processing_metadata(&processing_id)
                .into_iter()
                .find(|metadata| metadata.identity == identity);
            assert_eq!(registry_metadata, selected_metadata);
        }
    }
}

fn assert_has_identity(fields: &FieldDefSet, identity: &str) {
    assert!(
        fields
            .schema_metadata()
            .into_iter()
            .any(|metadata| metadata.identity.as_str() == identity),
        "missing identity {identity}"
    );
}

fn assert_lacks_identity(fields: &FieldDefSet, identity: &str) {
    assert!(
        fields
            .schema_metadata()
            .into_iter()
            .all(|metadata| metadata.identity.as_str() != identity),
        "unexpected identity {identity}"
    );
}

fn operation_key(operation: Operation) -> &'static str {
    match operation {
        Operation::Outline => "outline",
        Operation::Read => "read",
        Operation::Find => "find",
        Operation::Info => "info",
    }
}

fn ordered_registry<'a>(
    first_adapter: &'a TestAdapter,
    second_adapter: &'a TestAdapter,
) -> TestRegistry<'a> {
    TestRegistry {
        definitions: vec![
            definition(
                first_adapter,
                vec![
                    option("first", "outline", "--first-outline", Operation::Outline),
                    option("first", "read", "--first-read", Operation::Read),
                    option("first", "find", "--first-find", Operation::Find),
                    option("first", "info", "--first-info", Operation::Info),
                ],
            ),
            definition(
                second_adapter,
                vec![
                    option("second", "outline", "--second-outline", Operation::Outline),
                    option("second", "read", "--second-read", Operation::Read),
                    option("second", "find", "--second-find", Operation::Find),
                    option("second", "info", "--second-info", Operation::Info),
                ],
            ),
        ],
    }
}

fn option(owner: &str, key: &str, flag: &str, operation: Operation) -> AdapterOptionSpec {
    option_with_config_owner(owner, key, flag, operation, owner)
}

fn option_with_config_owner(
    owner: &str,
    key: &str,
    flag: &str,
    operation: Operation,
    config_owner: &str,
) -> AdapterOptionSpec {
    AdapterOptionSpec::builder(format!("docnav.adapters.{owner}.options.{key}"))
        .owner(owner)
        .operations(&[operation])
        .path(["options", key])
        .process(
            "cli",
            AdapterOptionProcessStrategy::cli_flag(flag).cli_metadata(
                CliProcessingMetadata::new()
                    .help(format!("Set {key}"))
                    .value_name("value"),
            ),
        )
        .process(
            "config",
            AdapterOptionProcessStrategy::config_path(["options", config_owner, key]),
        )
        .validation(FieldValidation::string().length(FieldLength::min(FieldBound::closed(1))))
        .default_static("default")
        .build()
}

fn definition<'a>(
    adapter: &'a TestAdapter,
    options: Vec<AdapterOptionSpec>,
) -> AdapterDefinition<'a> {
    AdapterDefinition::builder(adapter.adapter_id())
        .adapter(adapter)
        .manifest(adapter.manifest())
        .required_operation_handlers()
        .native_options(options)
        .build()
        .expect("valid test adapter definition")
}

struct TestRegistry<'a> {
    definitions: Vec<AdapterDefinition<'a>>,
}

impl NavigationAdapterRegistry for TestRegistry<'_> {
    fn adapters(&self) -> Vec<NavigationAdapterRef<'_>> {
        self.definitions
            .iter()
            .cloned()
            .map(NavigationAdapterRef::new)
            .collect()
    }
}

struct TestAdapter(&'static str);

impl Adapter for TestAdapter {
    fn adapter_id(&self) -> &str {
        self.0
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: "0.1".to_owned(),
            adapter: AdapterIdentity {
                id: self.0.to_owned(),
                name: self.0.to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: self.0.to_owned(),
                extensions: vec![format!(".{}", self.0)],
                content_types: vec![format!("text/{}", self.0)],
            }],
        }
    }

    fn probe(&self, _path: &str) -> ProbeResult {
        unreachable!("registry field tests do not probe adapters")
    }

    fn outline(
        &self,
        _request: &RequestEnvelope,
        _arguments: &OutlineArguments,
    ) -> AdapterResult<OutlineResult> {
        unreachable!("registry field tests do not dispatch adapters")
    }

    fn read(
        &self,
        _request: &RequestEnvelope,
        _arguments: &ReadArguments,
    ) -> AdapterResult<ReadResult> {
        unreachable!("registry field tests do not dispatch adapters")
    }

    fn find(
        &self,
        _request: &RequestEnvelope,
        _arguments: &FindArguments,
    ) -> AdapterResult<FindResult> {
        unreachable!("registry field tests do not dispatch adapters")
    }

    fn info(
        &self,
        _request: &RequestEnvelope,
        _arguments: &InfoArguments,
    ) -> AdapterResult<InfoResult> {
        unreachable!("registry field tests do not dispatch adapters")
    }
}
