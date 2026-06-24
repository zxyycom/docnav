## ADDED Requirements

### Requirement: CLI argv frontend delegates parameter semantics
The direct CLI argv frontend MUST classify argv tokens and map them to entrypoint metadata while delegating parameter semantics, defaults, operation applicability, and strict value validation to the standard parameter flow or the owning native option handler.

#### Scenario: Unused known flags do not fail before operation consumption
- **WHEN** a direct CLI invocation includes a known flag that is not consumed by the selected operation
- **THEN** the argv frontend does not eagerly validate that flag as an operation parameter
- **THEN** the entrypoint can report the existing compatible warning behavior
