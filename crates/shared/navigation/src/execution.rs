use docnav_adapter_contracts::{AdapterDefinition, AdapterDocument, StandardOperationInput};
use docnav_protocol::{Operation, ProtocolResponse, RequestEnvelope};

use crate::outline_mode::{execute_unstructured_outline, resolve_outline_mode, OutlineMode};
use crate::parameters::{
    resolve_adapter_intent, resolve_operation_input, AdapterIntent, ResolvedNavigationInput,
};
use crate::routing::{
    select_adapter, AdapterSelection, AdapterSelectionRequest, NavigationAdapterRegistry,
};
use crate::{
    auto_read, execute_protocol_request, protocol_request, AutoReadMode, DocumentParameterCatalog,
    NavigationCommand, NavigationCommandOutcome, NavigationConfigSources, NavigationError,
    NavigationFailureLayer, NavigationInputError, NavigationInvocationTrace, NavigationOutputMode,
    OperationInput,
};

pub(super) fn execute_loaded_navigation_command<R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
    catalog: &DocumentParameterCatalog,
    registry: &R,
) -> Result<NavigationCommandOutcome, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let document_path = command.document_path.clone();
    let prepared = prepare_loaded_navigation_command(command, config_sources, registry)?;
    execute_prepared_navigation_command(prepared, document_path, catalog)
}

pub struct PreparedNavigationSelection<'a> {
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
    selection: AdapterSelection<'a>,
    trace: NavigationInvocationTrace,
}

pub(super) fn prepare_loaded_navigation_command<'a, R>(
    command: NavigationCommand,
    config_sources: NavigationConfigSources,
    registry: &'a R,
) -> Result<PreparedNavigationSelection<'a>, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let mut trace = navigation_trace(command.operation);
    let adapter_intent = resolve_navigation_adapter_intent(&command, &config_sources, &mut trace)?;
    let selection = select_navigation_adapter(&command, &adapter_intent, registry, &mut trace)?;

    Ok(PreparedNavigationSelection {
        command,
        config_sources,
        selection,
        trace,
    })
}

pub(super) fn execute_prepared_navigation_command(
    mut prepared: PreparedNavigationSelection<'_>,
    document_path: String,
    catalog: &DocumentParameterCatalog,
) -> Result<NavigationCommandOutcome, NavigationError> {
    prepared.command.document_path = document_path;
    let PreparedNavigationSelection {
        command,
        config_sources,
        selection,
        mut trace,
    } = prepared;
    let resolved =
        resolve_navigation_input(&command, &config_sources, &selection, catalog, &mut trace)?;
    let prepared = prepare_navigation_request(command.operation, resolved, &mut trace)?;
    let mut document = selection
        .adapter
        .create_document(prepared.request.document.path.clone());
    let response = dispatch_navigation_request(
        &config_sources,
        &selection,
        document.as_mut(),
        &prepared,
        &mut trace,
    )?;
    let response = validate_navigation_response(response, &mut trace)?;
    let response = auto_read::compose_response(
        prepared.auto_read,
        document.as_mut(),
        &prepared.standard_input,
        response,
    );

    Ok(NavigationCommandOutcome {
        response,
        output: prepared.output,
        trace,
    })
}

struct PreparedNavigationRequest {
    request: RequestEnvelope,
    output: NavigationOutputMode,
    auto_read: Option<AutoReadMode>,
    standard_input: StandardOperationInput,
}

fn navigation_trace(operation: Operation) -> NavigationInvocationTrace {
    NavigationInvocationTrace {
        operation,
        selected_adapter_id: None,
        request_id: None,
        failure_layer: None,
    }
}

fn resolve_navigation_adapter_intent(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    trace: &mut NavigationInvocationTrace,
) -> Result<AdapterIntent, NavigationError> {
    resolve_adapter_intent(command, config_sources)
        .map_err(|error| error_with_trace(trace, NavigationFailureLayer::Config, error))
}

fn select_navigation_adapter<'a, R>(
    command: &NavigationCommand,
    adapter_intent: &AdapterIntent,
    registry: &'a R,
    trace: &mut NavigationInvocationTrace,
) -> Result<AdapterSelection<'a>, NavigationError>
where
    R: NavigationAdapterRegistry + ?Sized,
{
    let selection = select_adapter(AdapterSelectionRequest {
        registry,
        document_path: &command.document_path,
        preselected_adapter_id: adapter_intent.adapter_id.as_deref(),
        preselected_adapter_source: adapter_intent.source,
    })
    .map_err(|error| error_with_trace(trace, NavigationFailureLayer::AdapterSelection, error))?;
    trace.selected_adapter_id = Some(selection.adapter.id().to_owned());
    Ok(selection)
}

fn resolve_navigation_input(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selection: &AdapterSelection<'_>,
    catalog: &DocumentParameterCatalog,
    trace: &mut NavigationInvocationTrace,
) -> Result<ResolvedNavigationInput, NavigationError> {
    resolve_operation_input(command, config_sources, selection.adapter.id(), catalog).map_err(
        |error| error_with_trace(trace, NavigationFailureLayer::RequestConstruction, error),
    )
}

fn prepare_navigation_request(
    operation: Operation,
    resolved: ResolvedNavigationInput,
    trace: &mut NavigationInvocationTrace,
) -> Result<PreparedNavigationRequest, NavigationError> {
    let ResolvedNavigationInput {
        document_path,
        ref_id,
        query,
        page,
        limit,
        output,
        auto_read,
        options,
        standard_input,
    } = resolved;
    let request = protocol_request(OperationInput {
        operation,
        document_path,
        ref_id,
        query,
        page,
        limit,
        options,
    })
    .map_err(|error| input_error_with_trace(trace, error))?;
    trace.request_id = Some(request.request_id.clone());

    Ok(PreparedNavigationRequest {
        request,
        output,
        auto_read,
        standard_input,
    })
}

fn dispatch_navigation_request(
    config_sources: &NavigationConfigSources,
    selection: &AdapterSelection<'_>,
    document: &mut dyn AdapterDocument,
    prepared: &PreparedNavigationRequest,
    trace: &mut NavigationInvocationTrace,
) -> Result<ProtocolResponse, NavigationError> {
    let response = execute_navigation_request(
        config_sources,
        &selection.adapter,
        document,
        &prepared.request,
        &prepared.standard_input,
    )
    .map_err(|error| error_with_trace(trace, NavigationFailureLayer::AdapterDispatch, error))?;
    if matches!(response, ProtocolResponse::Failure(_)) {
        trace.failure_layer = Some(NavigationFailureLayer::AdapterDispatch);
    }
    Ok(response)
}

fn input_error_with_trace(
    trace: &mut NavigationInvocationTrace,
    error: NavigationInputError,
) -> NavigationError {
    trace.failure_layer = Some(NavigationFailureLayer::RequestConstruction);
    NavigationError::invalid_request(error.field(), error.reason()).with_invocation_trace(trace)
}

fn error_with_trace(
    trace: &mut NavigationInvocationTrace,
    layer: NavigationFailureLayer,
    error: NavigationError,
) -> NavigationError {
    trace.failure_layer = Some(layer);
    error.with_invocation_trace(trace)
}

pub(super) fn validate_navigation_response(
    response: ProtocolResponse,
    trace: &mut NavigationInvocationTrace,
) -> Result<ProtocolResponse, NavigationError> {
    response.validate().map_err(|error| {
        trace.failure_layer = Some(NavigationFailureLayer::ResultValidation);
        NavigationError::protocol_response_invalid(error.to_string()).with_invocation_trace(trace)
    })?;
    if auto_read::base_response_has_auto_read(&response) {
        trace.failure_layer = Some(NavigationFailureLayer::ResultValidation);
        return Err(NavigationError::protocol_response_invalid(
            "adapter base result contains navigation-owned auto_read",
        )
        .with_invocation_trace(trace));
    }
    Ok(response)
}

fn execute_navigation_request(
    config_sources: &NavigationConfigSources,
    adapter: &AdapterDefinition<'_>,
    document: &mut dyn AdapterDocument,
    request: &RequestEnvelope,
    standard_input: &StandardOperationInput,
) -> Result<ProtocolResponse, NavigationError> {
    if request.operation == Operation::Outline {
        if let OutlineMode::UnstructuredFull(unstructured) =
            resolve_outline_mode(config_sources, adapter.id(), adapter, document, request)?
        {
            return Ok(execute_unstructured_outline(
                adapter,
                document,
                request,
                unstructured,
            ));
        }
    }

    Ok(execute_protocol_request(document, request, standard_input))
}
