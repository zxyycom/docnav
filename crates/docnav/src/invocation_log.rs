mod content;
mod event;
mod hash;
mod paths;
mod settings;
mod summary;
mod time;
mod writer;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use docnav_navigation::{NavigationCommandOutcome, NavigationError, NavigationFailureLayer};
use docnav_protocol::{generate_request_id, Operation, ProtocolResponse, SuccessResponse};
use serde_json::{json, Value};

use crate::cli::DocumentCommand;
use crate::config::CoreConfig;
use crate::error::AppError;
use crate::project_context::ProjectContext;

use self::content::{capture_content_event, content_reference_for_result, result_page};
use self::event::{operation_event_base, response_size_bytes, OperationEvent};
use self::settings::InvocationLogSettings;
use self::summary::{
    app_error_summary, arguments_summary, core_error_summary, document_summary, failure_summary,
    protocol_code_text,
};
use self::writer::append_json_line;

const SCHEMA_VERSION: &str = "0.1";

#[derive(Clone, Debug)]
pub(crate) struct InvocationLogger {
    sink_path: Option<PathBuf>,
    content_capture_root: Option<PathBuf>,
    correlation_id: String,
}

#[derive(Clone, Debug)]
pub(crate) struct DocumentLogContext {
    operation: Operation,
    document: Value,
    arguments: Value,
}

#[derive(Clone, Debug)]
pub(crate) struct DocumentInvocationLog {
    logger: InvocationLogger,
    context: DocumentLogContext,
    started: Instant,
}

impl DocumentInvocationLog {
    pub(crate) fn new(
        logger: InvocationLogger,
        context: DocumentLogContext,
        started: Instant,
    ) -> Self {
        Self {
            logger,
            context,
            started,
        }
    }

    pub(crate) fn record_outcome(&self, outcome: &NavigationCommandOutcome) {
        self.logger
            .record_outcome(&self.context, outcome, self.started.elapsed());
    }

    pub(crate) fn record_output_projection_error(
        &self,
        outcome: &NavigationCommandOutcome,
        code: &str,
        summary: impl AsRef<str>,
    ) {
        self.logger.record_output_projection_error(
            &self.context,
            OutputProjectionFailure {
                outcome,
                code,
                summary: summary.as_ref().to_owned(),
                duration: self.started.elapsed(),
            },
        );
    }
}

impl InvocationLogger {
    pub(crate) fn explicit_cli(command: &DocumentCommand, project: &ProjectContext) -> Self {
        let settings = InvocationLogSettings::from_explicit_cli(command, project);
        Self {
            sink_path: settings.sink_path,
            content_capture_root: settings.content_capture_root,
            correlation_id: generate_request_id(),
        }
    }

    pub(crate) fn from_command(
        command: &DocumentCommand,
        project: &ProjectContext,
        project_config: &CoreConfig,
        user_config: &CoreConfig,
    ) -> Self {
        let settings =
            InvocationLogSettings::resolve(command, project, project_config, user_config);
        Self {
            sink_path: settings.sink_path,
            content_capture_root: settings.content_capture_root,
            correlation_id: generate_request_id(),
        }
    }

    pub(crate) fn enabled(&self) -> bool {
        self.sink_path.is_some()
    }

    pub(crate) fn document_context(
        &self,
        command: &DocumentCommand,
        project: &ProjectContext,
        absolute_path: Option<&Path>,
    ) -> DocumentLogContext {
        DocumentLogContext {
            operation: command.operation,
            document: document_summary(project, &command.path, absolute_path),
            arguments: arguments_summary(command),
        }
    }

    pub(crate) fn record_outcome(
        &self,
        context: &DocumentLogContext,
        outcome: &NavigationCommandOutcome,
        duration: Duration,
    ) {
        if !self.enabled() {
            return;
        }

        match &outcome.response {
            ProtocolResponse::Success(success) => {
                self.record_success(context, outcome, success, duration);
            }
            ProtocolResponse::Failure(failure) => {
                let layer = outcome
                    .trace
                    .failure_layer
                    .unwrap_or(NavigationFailureLayer::AdapterDispatch);
                let failure_summary = failure_summary(
                    layer.as_str(),
                    Some(protocol_code_text(failure.error.code())),
                    failure.error.message(),
                );
                let event = operation_event_base(
                    context,
                    OperationEvent {
                        name: "operation_failed",
                        status: "failure",
                        correlation_id: &self.correlation_id,
                        request_id: outcome.trace.request_id.as_deref(),
                        adapter_id: outcome.trace.selected_adapter_id.as_deref(),
                        duration,
                    },
                )
                .with_field("failure", failure_summary);
                self.append_event(event);
            }
        }
    }

    pub(crate) fn record_navigation_error(
        &self,
        context: &DocumentLogContext,
        error: &NavigationError,
        duration: Duration,
    ) {
        if !self.enabled() {
            return;
        }

        let event = operation_event_base(
            context,
            OperationEvent {
                name: "operation_failed",
                status: "failure",
                correlation_id: &self.correlation_id,
                request_id: error.request_id(),
                adapter_id: error.selected_adapter_id(),
                duration,
            },
        )
        .with_field(
            "failure",
            app_error_summary(
                error
                    .failure_layer()
                    .unwrap_or(NavigationFailureLayer::AdapterDispatch),
                error.diagnostic(),
            ),
        );
        self.append_event(event);
    }

    pub(crate) fn record_app_error(
        &self,
        context: &DocumentLogContext,
        error: &AppError,
        layer: &str,
        duration: Duration,
    ) {
        if !self.enabled() {
            return;
        }

        let event = operation_event_base(
            context,
            OperationEvent {
                name: "operation_failed",
                status: "failure",
                correlation_id: &self.correlation_id,
                request_id: None,
                adapter_id: None,
                duration,
            },
        )
        .with_field("failure", core_error_summary(layer, error));
        self.append_event(event);
    }

    fn record_output_projection_error(
        &self,
        context: &DocumentLogContext,
        failure: OutputProjectionFailure<'_>,
    ) {
        if !self.enabled() {
            return;
        }

        let event = operation_event_base(
            context,
            OperationEvent {
                name: "operation_failed",
                status: "failure",
                correlation_id: &self.correlation_id,
                request_id: failure.outcome.trace.request_id.as_deref(),
                adapter_id: failure.outcome.trace.selected_adapter_id.as_deref(),
                duration: failure.duration,
            },
        )
        .with_field(
            "failure",
            failure_summary(
                "output_projection",
                Some(failure.code.to_owned()),
                &failure.summary,
            ),
        );
        self.append_event(event);
    }

    fn record_success(
        &self,
        context: &DocumentLogContext,
        outcome: &NavigationCommandOutcome,
        success: &SuccessResponse,
        duration: Duration,
    ) {
        let content = content_reference_for_result(&success.result);
        let mut result = json!({
            "output_status": "ok",
            "response_size_bytes": response_size_bytes(&outcome.response),
        });
        if let Some(page) = result_page(&success.result) {
            result["page"] = json!(page.get());
        }
        if let Some(reference) = content.as_ref() {
            result["content"] = reference.value.clone();
        }

        let event = operation_event_base(
            context,
            OperationEvent {
                name: "operation_completed",
                status: "success",
                correlation_id: &self.correlation_id,
                request_id: outcome.trace.request_id.as_deref(),
                adapter_id: outcome.trace.selected_adapter_id.as_deref(),
                duration,
            },
        )
        .with_field("result", result);
        self.append_event(event);

        if let (Some(root), Some(content)) = (&self.content_capture_root, content) {
            let event = capture_content_event(
                root,
                &self.correlation_id,
                context,
                outcome.trace.request_id.as_deref(),
                &content,
            );
            self.append_event(event);
        }
    }

    fn append_event(&self, event: Value) {
        let Some(path) = &self.sink_path else {
            return;
        };
        let _ = append_json_line(path, &event);
    }
}

struct OutputProjectionFailure<'a> {
    outcome: &'a NavigationCommandOutcome,
    code: &'a str,
    summary: String,
    duration: Duration,
}
