pub(in crate::direct::args) mod flags {
    pub(in crate::direct::args) const LIMIT: &str = "--limit";
    pub(in crate::direct::args) const OUTPUT: &str = "--output";
    pub(in crate::direct::args) const PAGE: &str = "--page";
    pub(in crate::direct::args) const PAGINATION: &str = "--pagination";
    pub(in crate::direct::args) const PROJECT_CONFIG_PATH: &str = "--project-config-path";
    pub(in crate::direct::args) const QUERY: &str = "--query";
    pub(in crate::direct::args) const REF: &str = "--ref";
    pub(in crate::direct::args) const USER_CONFIG_PATH: &str = "--user-config-path";
}

pub(in crate::direct::args) mod arg_ids {
    pub(in crate::direct::args) const LIMIT: &str = "limit";
    pub(in crate::direct::args) const OUTPUT: &str = "output";
    pub(in crate::direct::args) const PAGE: &str = "page";
    pub(in crate::direct::args) const PAGINATION: &str = "pagination";
    pub(in crate::direct::args) const PATH: &str = "path";
    pub(in crate::direct::args) const PROJECT_CONFIG_PATH: &str = "project_config_path";
    pub(in crate::direct::args) const QUERY: &str = "query";
    pub(in crate::direct::args) const REF: &str = "ref";
    pub(in crate::direct::args) const USER_CONFIG_PATH: &str = "user_config_path";
}

pub(in crate::direct) mod command_names {
    pub(crate) const FIND: &str = "find";
    pub(crate) const INFO: &str = "info";
    pub(crate) const INVOKE: &str = "invoke";
    pub(crate) const MANIFEST: &str = "manifest";
    pub(crate) const OUTLINE: &str = "outline";
    pub(crate) const PROBE: &str = "probe";
    pub(crate) const READ: &str = "read";
}

pub(super) mod defaults {
    use crate::direct::args::standard;

    pub(in crate::direct::args) const LIMIT: &str = standard::DEFAULT_LIMIT_TEXT;
    pub(in crate::direct::args) const LIMIT_VALUE: u32 = 6000;
    pub(in crate::direct::args) const OUTPUT: &str = standard::DEFAULT_OUTPUT_TEXT;
    pub(in crate::direct::args) const PAGE: &str = standard::DEFAULT_PAGE_TEXT;
    pub(in crate::direct::args) const PAGINATION: &str = "enabled";
    pub(in crate::direct::args) const PROTOCOL_OUTPUT: &str =
        standard::DEFAULT_PROTOCOL_OUTPUT_TEXT;
}

pub(super) mod output_values {
    pub(in crate::direct::args) const PROTOCOL_JSON: &str = "protocol-json";
    pub(in crate::direct::args) const READABLE_JSON: &str = "readable-json";
    pub(in crate::direct::args) const READABLE_VIEW: &str = "readable-view";
}

pub(in crate::direct::args) mod pagination_values {
    pub(in crate::direct::args) const DISABLED: &str = "disabled";
    pub(in crate::direct::args) const ENABLED: &str = "enabled";
}

pub(in crate::direct::args) mod command_labels {
    pub(in crate::direct::args) const MANIFEST: &str = "manifest";
    pub(in crate::direct::args) const PROBE: &str = "probe";
}

pub(in crate::direct::args) mod input_errors {
    pub(in crate::direct::args) const PROTOCOL_OUTPUT_ONLY: &str =
        "only --output protocol-json is supported for this command";
}
