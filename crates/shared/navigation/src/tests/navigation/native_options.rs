mod adapter_scopes;
mod defaults;
// @case WB-NAV-INPUT-RESOLUTION-001
mod resolution;
mod validation;

use docnav_adapter_contracts::StandardOperationInput;
use docnav_protocol::{Operation, OperationResult, ProtocolResponse};
use serde_json::{json, Value};

use crate::{
    execute_loaded_navigation_command, resolve_operation_input, NavigationAdapterRegistry,
    NavigationOutputMode,
};

use super::super::support::{
    cli_value_candidate, config_sources, navigation_command, MultiAdapterRegistry, StubRegistry,
};
