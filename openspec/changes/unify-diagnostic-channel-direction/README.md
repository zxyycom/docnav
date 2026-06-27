# unify-diagnostic-channel-direction

Record the target direction for a forced migration to a unified internal `DiagnosticStack`. All Docnav diagnostics and errors move through `docnav-diagnostics`: push recoverable warnings, skipped-condition diagnostics, and fatal contexts into an identifiable LIFO stack, then let each caller or surface owner decide how to interpret and output stack entries. Existing `StableError`, warning, and direct stderr paths are migration targets, not compatibility boundaries.
