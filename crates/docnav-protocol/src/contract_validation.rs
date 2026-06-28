mod enums;
mod field_builders;
mod formatting;
mod helpers;
mod manifest;
mod probe;
mod request;
mod response;
mod response_fields;
mod response_results;

pub(crate) use manifest::validate_manifest_contract_value;
pub(crate) use probe::validate_probe_result_contract_value;
pub(crate) use request::validate_protocol_request_contract_value;
pub(crate) use response::validate_protocol_response_contract_value;

const JSON_CONTRACT_PROCESSING: &str = "json-contract";
const VALUE_FIELD: &str = "value";
