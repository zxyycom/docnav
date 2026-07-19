use docnav_adapter_contracts::StandardInputBinding;
use docnav_navigation::{
    AutoReadMode, DocumentParameterBinding, DocumentParameterCatalog,
    DocumentParameterCatalogBuildError, DocumentParameterEntry, NavigationOutputMode,
};
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{
    CliBooleanEncoding, CliProcessingMetadata, ExpectedFieldShape, FieldBound, FieldDef,
    FieldDefSet, FieldDefSetBuilder, FieldIdentity, FieldValidation, MergeStrategy,
    ProcessStrategy,
};

use crate::registry::AdapterRegistry;

pub(crate) const PAGE_IDENTITY: &str = "docnav.document.page";
pub(crate) const LIMIT_IDENTITY: &str = "docnav.defaults.pagination.limit";
pub(crate) const PAGINATION_ENABLED_IDENTITY: &str = "docnav.defaults.pagination.enabled";
pub(crate) const OUTPUT_IDENTITY: &str = "docnav.defaults.output";
pub(crate) const AUTO_READ_IDENTITY: &str = "docnav.defaults.auto_read";
pub(crate) const MAX_HEADING_LEVEL_IDENTITY: &str =
    "docnav.adapters.docnav-markdown.options.max_heading_level";

const CLI_PROCESSING: &str = "cli";
const CONFIG_PROCESSING: &str = "config";
const MARKDOWN_ADAPTER_ID: &str = "docnav-markdown";
const MAX_PAGINATION_LIMIT: i64 = u32::MAX as i64;

pub(crate) fn document_parameter_catalog(
) -> Result<DocumentParameterCatalog, DocumentParameterCatalogBuildError> {
    let registry = AdapterRegistry::builtin();

    DocumentParameterCatalog::new(
        registry
            .adapters
            .iter()
            .map(|definition| definition().id().to_owned()),
        document_parameter_fields(),
        document_parameter_entries(),
    )
}

fn document_parameter_fields() -> FieldDefSetBuilder {
    FieldDefSet::builder()
        .field(
            FieldDef::builder(PAGE_IDENTITY)
                .process(
                    CLI_PROCESSING,
                    ProcessStrategy::cli_flag("--page").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Select the result page")
                            .value_name("positive integer"),
                    ),
                )
                .validation(positive_integer_validation())
                .default_static(1)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(LIMIT_IDENTITY)
                .process(
                    CLI_PROCESSING,
                    ProcessStrategy::cli_flag("--limit").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Set the result page size")
                            .value_name("positive integer"),
                    ),
                )
                .process(
                    CONFIG_PROCESSING,
                    ProcessStrategy::config_path(["defaults", "pagination", "limit"]),
                )
                .validation(positive_integer_validation())
                .default_static(6000)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(PAGINATION_ENABLED_IDENTITY)
                .process(
                    CLI_PROCESSING,
                    ProcessStrategy::cli_flag("--pagination").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Enable or disable pagination")
                            .value_name("enabled|disabled")
                            .boolean_encoding(CliBooleanEncoding::explicit("enabled", "disabled")),
                    ),
                )
                .process(
                    CONFIG_PROCESSING,
                    ProcessStrategy::config_path(["defaults", "pagination", "enabled"]),
                )
                .validation(FieldValidation::boolean())
                .default_static(true)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(OUTPUT_IDENTITY)
                .process(
                    CLI_PROCESSING,
                    ProcessStrategy::cli_flag("--output").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Select the document output mode")
                            .value_name("mode"),
                    ),
                )
                .process(
                    CONFIG_PROCESSING,
                    ProcessStrategy::config_path(["defaults", "output"]),
                )
                .validation(FieldValidation::string_enum::<NavigationOutputMode>())
                .default_static(NavigationOutputMode::ReadableView)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(AUTO_READ_IDENTITY)
                .process(
                    CLI_PROCESSING,
                    ProcessStrategy::cli_flag("--auto-read").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Automatically read a unique returned ref")
                            .value_name("disabled|unique-ref"),
                    ),
                )
                .process(
                    CONFIG_PROCESSING,
                    ProcessStrategy::config_path(["defaults", "auto_read"]),
                )
                .validation(FieldValidation::string_enum::<AutoReadMode>())
                .default_static(AutoReadMode::UniqueRef)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(MAX_HEADING_LEVEL_IDENTITY)
                .process(
                    CLI_PROCESSING,
                    ProcessStrategy::cli_flag("--max-heading-level").cli_metadata(
                        CliProcessingMetadata::new()
                            .help("Set the maximum Markdown heading level")
                            .value_name("value"),
                    ),
                )
                .process(
                    CONFIG_PROCESSING,
                    ProcessStrategy::config_path([
                        "options",
                        MARKDOWN_ADAPTER_ID,
                        "max_heading_level",
                    ]),
                )
                .validation(
                    FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
                )
                .default_static(3)
                .merge(MergeStrategy::Replace),
            ExpectedFieldShape::optional(),
        )
}

fn document_parameter_entries() -> Vec<DocumentParameterEntry> {
    vec![
        entry(
            PAGE_IDENTITY,
            None,
            [
                DocumentParameterBinding::StandardInput(StandardInputBinding::OutlinePage),
                DocumentParameterBinding::StandardInput(StandardInputBinding::ReadPage),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindPage),
            ],
        ),
        entry(
            LIMIT_IDENTITY,
            None,
            [
                DocumentParameterBinding::StandardInput(StandardInputBinding::OutlineLimit),
                DocumentParameterBinding::StandardInput(StandardInputBinding::ReadLimit),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindLimit),
            ],
        ),
        entry(
            PAGINATION_ENABLED_IDENTITY,
            None,
            [
                DocumentParameterBinding::PaginationEnabled(PagedOperation::Outline),
                DocumentParameterBinding::PaginationEnabled(PagedOperation::Read),
                DocumentParameterBinding::PaginationEnabled(PagedOperation::Find),
            ],
        ),
        entry(
            OUTPUT_IDENTITY,
            None,
            [
                DocumentParameterBinding::OutputMode(Operation::Outline),
                DocumentParameterBinding::OutputMode(Operation::Read),
                DocumentParameterBinding::OutputMode(Operation::Find),
                DocumentParameterBinding::OutputMode(Operation::Info),
            ],
        ),
        entry(
            AUTO_READ_IDENTITY,
            None,
            [
                DocumentParameterBinding::AutoReadMode(Operation::Outline),
                DocumentParameterBinding::AutoReadMode(Operation::Find),
            ],
        ),
        entry(
            MAX_HEADING_LEVEL_IDENTITY,
            Some(MARKDOWN_ADAPTER_ID),
            [
                DocumentParameterBinding::StandardInput(
                    StandardInputBinding::OutlineMaxHeadingLevel,
                ),
                DocumentParameterBinding::StandardInput(StandardInputBinding::FindMaxHeadingLevel),
            ],
        ),
    ]
}

fn positive_integer_validation() -> FieldValidation<i64> {
    FieldValidation::int().between(
        FieldBound::closed(1),
        FieldBound::closed(MAX_PAGINATION_LIMIT),
    )
}

fn entry<const N: usize>(
    identity: &str,
    adapter_id: Option<&str>,
    bindings: [DocumentParameterBinding; N],
) -> DocumentParameterEntry {
    DocumentParameterEntry::new(
        FieldIdentity::new(identity).expect("core parameter identity must be valid"),
        adapter_id.map(str::to_owned),
        bindings.into(),
    )
}

#[cfg(test)]
mod tests;
