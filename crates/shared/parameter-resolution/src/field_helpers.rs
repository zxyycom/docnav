use docnav_typed_fields::{
    FieldBound, FieldDef, FieldDefBuilder, FieldLength, FieldStringEnum, FieldValidation,
    ProcessStrategy,
};

pub mod ids {
    pub const ADAPTER: &str = "docnav.defaults.adapter";
    pub const INVOCATION_LOG_CONTENT_CAPTURE_ENABLED: &str =
        "docnav.invocation_log.content_capture.enabled";
    pub const INVOCATION_LOG_CONTENT_CAPTURE_ROOT: &str =
        "docnav.invocation_log.content_capture.root";
    pub const INVOCATION_LOG_ENABLED: &str = "docnav.invocation_log.enabled";
    pub const INVOCATION_LOG_PATH: &str = "docnav.invocation_log.path";
    pub const LIMIT: &str = "docnav.defaults.pagination.limit";
    pub const OUTPUT: &str = "docnav.defaults.output";
    pub const PAGE: &str = "docnav.document.page";
    pub const PAGINATION_ENABLED: &str = "docnav.defaults.pagination.enabled";
    pub const PATH: &str = "docnav.document.path";
    pub const QUERY: &str = "docnav.document.query";
    pub const REF: &str = "docnav.document.ref";
}

pub const MAX_PAGINATION_LIMIT: u32 = u32::MAX;

pub fn document_path_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::PATH, processing_id, ["path"])
}

pub fn read_ref_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::REF, processing_id, ["ref"])
}

pub fn find_query_field(processing_id: &'static str) -> FieldDefBuilder<String> {
    direct_string_field(ids::QUERY, processing_id, ["query"])
}

pub fn adapter_id_field(
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
            ProcessStrategy::json_path(["defaults", "adapter"]),
        )
        .validation(non_empty_string_validation())
}

pub fn page_field(processing_id: &'static str) -> FieldDefBuilder<i64> {
    direct_positive_u32_field(ids::PAGE, processing_id, ["page"])
}

pub fn limit_field(processing_id: &'static str) -> FieldDefBuilder<i64> {
    direct_positive_u32_field(ids::LIMIT, processing_id, ["limit"])
}

pub fn configurable_limit_field(
    direct_processing_id: &'static str,
    config_processing_id: &'static str,
) -> FieldDefBuilder<i64> {
    FieldDef::builder(ids::LIMIT)
        .process(direct_processing_id, ProcessStrategy::json_path(["limit"]))
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["defaults", "pagination", "limit"]),
        )
        .validation(positive_u32_int_validation())
}

pub fn pagination_enabled_field(
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
            ProcessStrategy::json_path(["defaults", "pagination", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

pub fn config_pagination_enabled_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::PAGINATION_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["defaults", "pagination", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

pub fn invocation_log_enabled_field(config_processing_id: &'static str) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::INVOCATION_LOG_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

pub fn invocation_log_path_field(config_processing_id: &'static str) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::INVOCATION_LOG_PATH)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "path"]),
        )
        .validation(non_empty_string_validation())
}

pub fn invocation_log_content_capture_enabled_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<bool> {
    FieldDef::builder(ids::INVOCATION_LOG_CONTENT_CAPTURE_ENABLED)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "content_capture", "enabled"]),
        )
        .validation(FieldValidation::boolean())
}

pub fn invocation_log_content_capture_root_field(
    config_processing_id: &'static str,
) -> FieldDefBuilder<String> {
    FieldDef::builder(ids::INVOCATION_LOG_CONTENT_CAPTURE_ROOT)
        .process(
            config_processing_id,
            ProcessStrategy::json_path(["invocation_log", "content_capture", "root"]),
        )
        .validation(non_empty_string_validation())
}

pub fn configurable_output_field<T>(
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
            ProcessStrategy::json_path(["defaults", "output"]),
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
        FieldBound::closed(i64::from(u32::MAX)),
    )
}
