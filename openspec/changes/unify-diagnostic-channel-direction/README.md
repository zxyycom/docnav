# unify-diagnostic-channel-direction

Record Docnav's target error-channel contract. Runtime and public surface code push problems into a request-local stack owned by `docnav-diagnostics`; all Docnav crates use `DiagnosticCode` from that crate. `DiagnosticCode` aggregates manually grouped code enums, owns each code's canonical details object, and is the only diagnostic identity source. Boundary surfaces read stack records and project them to protocol, readable output, stderr, and exit behavior according to their owner docs.
