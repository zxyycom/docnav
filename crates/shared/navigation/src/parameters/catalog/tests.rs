mod associations;
mod bindings;
mod projection;

use docnav_adapter_contracts::StandardInputBinding;
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldDefSetBuildError, FieldIdentity,
    FieldValidation, ProcessStrategy, ValueKind,
};
use serde_json::json;

use super::{
    DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterCatalogBuildError,
    DocumentParameterEntry,
};
use crate::{
    validate_navigation_config_source_value, NavigationConfigSourceLevel,
    NavigationConfigSourceOrigin,
};

const FIRST_IDENTITY: &str = "docnav.parameters.first";
const SECOND_IDENTITY: &str = "docnav.parameters.second";
const KNOWN_ADAPTER: &str = "known-adapter";
const SECOND_ADAPTER: &str = "second-adapter";

fn catalog<T: 'static>(
    field: docnav_typed_fields::FieldDefBuilder<T>,
    entries: Vec<DocumentParameterEntry>,
) -> Result<DocumentParameterCatalog, DocumentParameterCatalogBuildError> {
    DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        FieldDefSet::builder().field(field, ExpectedFieldShape::optional()),
        entries,
    )
}

fn integer_field(identity: &str, flag: &str) -> docnav_typed_fields::FieldDefBuilder<i64> {
    FieldDef::builder(identity)
        .process("cli", ProcessStrategy::cli_flag(flag))
        .validation(FieldValidation::int())
}

fn entry(
    identity: &str,
    adapter_id: Option<&str>,
    bindings: &[StandardInputBinding],
) -> DocumentParameterEntry {
    DocumentParameterEntry::new(
        FieldIdentity::new(identity).expect("test identity is valid"),
        adapter_id.map(str::to_owned),
        bindings
            .iter()
            .copied()
            .map(DocumentParameterBinding::StandardInput)
            .collect(),
    )
}
