use docnav_typed_fields::{
    CliBooleanEncoding, CliProcessingMetadata, FieldBound, FieldDef, FieldDefBuilder, FieldLength,
    FieldStringEnum, FieldValidation, ProcessStrategy,
};

use super::super::{ids, MAX_PAGINATION_LIMIT};

const CLI_PROCESSING: &str = "cli";

#[cfg(test)]
mod tests;

pub(super) fn document_path_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::PATH, processing_id, ["path"])
}

pub(super) fn read_ref_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::REF, processing_id, ["ref"])
}

pub(super) fn find_query_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::QUERY, processing_id, ["query"])
}

pub(super) fn adapter_id_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::ADAPTER)
        .process(
            direct_processing_id,
            ProcessStrategy::json_path(["adapter"]),
        )
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["defaults", "adapter"]),
        )
        .process(
            CLI_PROCESSING,
            ProcessStrategy::cli_flag("--adapter").cli_metadata(
                CliProcessingMetadata::new()
                    .help("Select the adapter for this document")
                    .value_name("adapter-id"),
            ),
        )
        .validation(non_empty_string_validation())
}

pub(super) fn standard_page_field(processing_id: &'static str) -> FieldDefBuilder<i64> {
    direct_positive_u32_field(ids::PAGE, processing_id, ["page"]).process(
        CLI_PROCESSING,
        ProcessStrategy::cli_flag("--page").cli_metadata(
            CliProcessingMetadata::new()
                .help("Select the result page")
                .value_name("positive integer"),
        ),
    )
}

pub(super) fn configurable_limit_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<i64> {
    FieldDef::builder(ids::LIMIT)
        .process(direct_processing_id, ProcessStrategy::json_path(["limit"]))
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["defaults", "pagination", "limit"]),
        )
        .process(
            CLI_PROCESSING,
            ProcessStrategy::cli_flag("--limit").cli_metadata(
                CliProcessingMetadata::new()
                    .help("Set the result page size")
                    .value_name("positive integer"),
            ),
        )
        .validation(positive_u32_int_validation())
}

pub(super) fn pagination_enabled_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::PAGINATION_ENABLED)
        .process(
            direct_processing_id,
            ProcessStrategy::json_path(["pagination"]),
        )
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["defaults", "pagination", "enabled"]),
        )
        .process(
            CLI_PROCESSING,
            ProcessStrategy::cli_flag("--pagination").cli_metadata(
                CliProcessingMetadata::new()
                    .help("Enable or disable pagination")
                    .value_name("enabled|disabled")
                    .boolean_encoding(CliBooleanEncoding::explicit("enabled", "disabled")),
            ),
        )
        .validation(FieldValidation::boolean())
}

pub(super) fn invocation_log_enabled_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::INVOCATION_LOG_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["invocation_log", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

pub(super) fn invocation_log_path_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::INVOCATION_LOG_PATH)
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["invocation_log", "path"]),
        )
        .validation(non_empty_string_validation())
}

pub(super) fn invocation_log_content_capture_enabled_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::INVOCATION_LOG_CONTENT_CAPTURE_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["invocation_log", "content_capture", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

pub(super) fn invocation_log_content_capture_root_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::INVOCATION_LOG_CONTENT_CAPTURE_ROOT)
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["invocation_log", "content_capture", "root"]),
        )
        .validation(non_empty_string_validation())
}

pub(super) fn configurable_output_field<T>(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<T>
where
    T: FieldStringEnum,
{
    FieldDef::builder(ids::OUTPUT)
        .process(direct_processing_id, ProcessStrategy::json_path(["output"]))
        .process(
            config_processing_id,
            ProcessStrategy::config_path(["defaults", "output"]),
        )
        .process(
            CLI_PROCESSING,
            ProcessStrategy::cli_flag("--output").cli_metadata(
                CliProcessingMetadata::new()
                    .help("Select the document output mode")
                    .value_name("mode"),
            ),
        )
        .validation(FieldValidation::string_enum::<T>())
}

fn direct_string_field<const N: usize>(
    identity: &str,
    processing_id: &'static str,
    direct_path: [&str; N],
) -> FieldDefBuilder<String> {
    FieldDef::builder(identity)
        .process(processing_id, ProcessStrategy::json_path(direct_path))
        .validation(non_empty_string_validation())
}

fn direct_positive_u32_field<const N: usize>(
    identity: &str,
    processing_id: &'static str,
    direct_path: [&str; N],
) -> FieldDefBuilder<i64> {
    FieldDef::builder(identity)
        .process(processing_id, ProcessStrategy::json_path(direct_path))
        .validation(positive_u32_int_validation())
}

fn non_empty_string_validation() -> FieldValidation<String> {
    FieldValidation::string().length(FieldLength::min(FieldBound::closed(1)))
}

fn positive_u32_int_validation() -> FieldValidation<i64> {
    FieldValidation::int().between(
        FieldBound::closed(1),
        FieldBound::closed(i64::from(MAX_PAGINATION_LIMIT)),
    )
}
