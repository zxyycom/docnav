use docnav_adapter_contracts::{
    FindInput, InfoInput, OutlineInput, ReadInput, StandardInputBinding, StandardOperationInput,
};
use docnav_protocol::{Operation, Options, PagedOperation, PositiveInteger};

use super::{
    config, fields, ids, resolve_command_with_fields, values, DocumentParameterBinding,
    DocumentParameterCatalog, InputResolutionContext, ResolvedNavigationInput,
};
use crate::{
    AutoReadMode, NavigationCommand, NavigationConfigSources, NavigationError, NavigationOutputMode,
};

pub(crate) fn resolve_operation_input(
    command: &NavigationCommand,
    config_sources: &NavigationConfigSources,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let operation_fields =
        fields::operation_fields(command.operation, selected_adapter_id, catalog)?;
    config::validate_navigation_sources(
        command,
        config_sources,
        selected_adapter_id,
        &operation_fields,
        catalog,
    )?;

    let resolution =
        resolve_command_with_fields(operation_fields.as_ref(), command, config_sources)?;

    config::first_operation_resolution_error(
        &resolution,
        config_sources,
        selected_adapter_id,
        command.operation,
        catalog,
    )?;

    resolved_input_from_resolution(
        command.operation,
        selected_adapter_id,
        catalog,
        operation_fields.as_ref(),
        &resolution,
    )
}

fn resolved_input_from_resolution(
    operation: Operation,
    selected_adapter_id: &str,
    catalog: &DocumentParameterCatalog,
    fields: &cli_config_resolution::FieldDefSet,
    resolution: &cli_config_resolution::ResolutionResult,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let context = InputResolutionContext {
        selected_adapter_id,
        catalog,
        fields,
        resolution,
    };
    let output =
        context.required_output_binding(DocumentParameterBinding::OutputMode(operation))?;

    match operation {
        Operation::Outline => resolved_outline_input(
            &context,
            output,
            context.required_auto_read_binding(DocumentParameterBinding::AutoReadMode(
                Operation::Outline,
            ))?,
        ),
        Operation::Read => resolved_read_input(&context, output),
        Operation::Find => resolved_find_input(
            &context,
            output,
            context.required_auto_read_binding(DocumentParameterBinding::AutoReadMode(
                Operation::Find,
            ))?,
        ),
        Operation::Info => resolved_info_input(&context, output),
    }
}

fn resolved_outline_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
    auto_read: AutoReadMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let page = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::OutlinePage,
    ))?;
    let raw_limit = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::OutlineLimit,
    ))?;
    let pagination_enabled = context.required_bool_binding(
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Outline),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let max_heading_binding = StandardInputBinding::OutlineMaxHeadingLevel;
    let max_heading_level = context.optional_adapter_integer_binding(max_heading_binding)?;
    let options = protocol_options(max_heading_binding, max_heading_level);
    let standard_input = StandardOperationInput::Outline(OutlineInput {
        document_path: document_path.clone(),
        page,
        limit,
        max_heading_level,
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: None,
        query: None,
        page: Some(page),
        limit: Some(limit),
        output,
        auto_read: Some(auto_read),
        options,
        standard_input,
    })
}

fn resolved_read_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let ref_id = context.required_string(ids::REF)?;
    let page = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::ReadPage,
    ))?;
    let raw_limit = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::ReadLimit,
    ))?;
    let pagination_enabled = context.required_bool_binding(
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Read),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let standard_input = StandardOperationInput::Read(ReadInput {
        document_path: document_path.clone(),
        ref_id: ref_id.clone(),
        page,
        limit,
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: Some(ref_id),
        query: None,
        page: Some(page),
        limit: Some(limit),
        output,
        auto_read: None,
        options: None,
        standard_input,
    })
}

fn resolved_find_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
    auto_read: AutoReadMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let query = context.required_string(ids::QUERY)?;
    let page = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::FindPage,
    ))?;
    let raw_limit = context.required_positive_binding(DocumentParameterBinding::StandardInput(
        StandardInputBinding::FindLimit,
    ))?;
    let pagination_enabled = context.required_bool_binding(
        DocumentParameterBinding::PaginationEnabled(PagedOperation::Find),
    )?;
    let limit = effective_limit(raw_limit, pagination_enabled);
    let max_heading_binding = StandardInputBinding::FindMaxHeadingLevel;
    let max_heading_level = context.optional_adapter_integer_binding(max_heading_binding)?;
    let options = protocol_options(max_heading_binding, max_heading_level);
    let standard_input = StandardOperationInput::Find(FindInput {
        document_path: document_path.clone(),
        query: query.clone(),
        page,
        limit,
        max_heading_level,
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: None,
        query: Some(query),
        page: Some(page),
        limit: Some(limit),
        output,
        auto_read: Some(auto_read),
        options,
        standard_input,
    })
}

fn resolved_info_input(
    context: &InputResolutionContext<'_>,
    output: NavigationOutputMode,
) -> Result<ResolvedNavigationInput, NavigationError> {
    let document_path = context.required_string(ids::PATH)?;
    let standard_input = StandardOperationInput::Info(InfoInput {
        document_path: document_path.clone(),
    });

    Ok(ResolvedNavigationInput {
        document_path,
        ref_id: None,
        query: None,
        page: None,
        limit: None,
        output,
        auto_read: None,
        options: None,
        standard_input,
    })
}

fn protocol_options(binding: StandardInputBinding, value: Option<i64>) -> Option<Options> {
    let key = protocol_option_key(binding)?;
    value.map(|value| Options::from_iter([(key.to_owned(), value.into())]))
}

fn protocol_option_key(binding: StandardInputBinding) -> Option<&'static str> {
    match binding {
        StandardInputBinding::OutlineMaxHeadingLevel
        | StandardInputBinding::FindMaxHeadingLevel => Some("max_heading_level"),
        StandardInputBinding::OutlinePage
        | StandardInputBinding::OutlineLimit
        | StandardInputBinding::ReadPage
        | StandardInputBinding::ReadLimit
        | StandardInputBinding::FindPage
        | StandardInputBinding::FindLimit => None,
    }
}

fn effective_limit(limit: PositiveInteger, pagination_enabled: bool) -> PositiveInteger {
    if pagination_enabled {
        limit
    } else {
        values::max_pagination_limit()
    }
}
