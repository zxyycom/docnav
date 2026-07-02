## ADDED Requirements

### Requirement: local service mode MUST NOT provide adapter implementation source
Local service mode MUST use the same core release static adapter registry and adapter library handles as the non-service path. It MUST NOT discover, host, launch, or fallback to external adapter executables.

#### Scenario: Service path uses static registry
- **WHEN** a document operation runs through a local service path
- **THEN** adapter selection uses the current core release static registry
- **THEN** the selected adapter implementation is the same adapter library handle used without service mode

### Requirement: local service mode MUST preserve document output contracts
Local service mode MUST NOT add service status, cache status, or internal protocol facts to successful `readable-view`, `readable-json`, or `protocol-json` document output.

#### Scenario: Service status is not part of success payload
- **WHEN** a document operation succeeds through a local service path
- **THEN** stdout matches the documented document output shape for the selected output mode
- **THEN** service/cache status is only available through an explicitly documented diagnostic, doctor, or status surface
