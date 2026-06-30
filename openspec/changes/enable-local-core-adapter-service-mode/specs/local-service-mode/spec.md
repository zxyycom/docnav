## ADDED Requirements

### Requirement: Local service mode provides default core and adapter fast path
Local service mode MUST provide a default-enabled local fast path that includes both core service behavior and adapter service behavior while preserving the existing single-request adapter invoke path as a fallback entrypoint.

#### Scenario: Connection failures fallback but internal protocol mismatches fail
- **WHEN** the default service path cannot connect to the local service endpoint
- **THEN** Docnav falls back to the existing invoke path and records an internal fast-path diagnostic or owner-scoped status
- **WHEN** the service path connects but handshake, wire hash, frame, or internal payload decoding is incompatible
- **THEN** Docnav returns a hard failure instead of falling back
