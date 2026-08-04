use std::io::Write;
use std::time::Instant;

use docnav_navigation::NavigationCommandOutcome;

use super::{DocumentLogContext, InvocationLogger, OutputProjectionFailure};

const WRITE_FAILURE_WARNING: &str =
    "docnav warning: unable to append invocation log; check the configured log path and permissions";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct InvocationLogDiagnostic;

impl InvocationLogDiagnostic {
    pub(crate) fn write_to(self, stderr: &mut impl Write) {
        let _ = writeln!(stderr, "{WRITE_FAILURE_WARNING}");
    }
}

#[derive(Debug)]
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

    pub(crate) fn record_outcome(
        &self,
        outcome: &NavigationCommandOutcome,
    ) -> Option<InvocationLogDiagnostic> {
        self.logger
            .record_outcome(&self.context, outcome, self.started.elapsed())
    }

    pub(crate) fn record_output_projection_error(
        &self,
        outcome: &NavigationCommandOutcome,
        code: &str,
        summary: impl AsRef<str>,
    ) -> Option<InvocationLogDiagnostic> {
        self.logger.record_output_projection_error(
            &self.context,
            OutputProjectionFailure {
                outcome,
                code,
                summary: summary.as_ref().to_owned(),
                duration: self.started.elapsed(),
            },
        )
    }
}
