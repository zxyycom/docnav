## ADDED Requirements

### Requirement: CLI argv frontend delegates parameter semantics
The `clap`-backed direct CLI argv frontend MUST classify argv tokens and map them to entrypoint metadata while delegating parameter semantics, defaults, operation applicability, and strict value validation to the standard parameter flow or the owning native option handler.

#### Scenario: Operation-inapplicable input fails at the entry boundary
- **WHEN** a direct CLI invocation includes unknown argv, extra positional input, or a known flag that is not applicable to the selected operation
- **THEN** the argv frontend does not pass that input to adapter execution
- **THEN** the entrypoint reports a primary input diagnostic under the strict direct CLI contract

#### Scenario: Consumed parameters remain strict
- **WHEN** a direct CLI invocation includes a parameter consumed by the selected operation
- **THEN** strict value validation is performed by the standard parameter flow or owning native option handler
- **THEN** invalid consumed values still fail through the strict input diagnostic path

### Requirement: CLI argv frontend supports metadata-driven help
The `clap`-backed direct CLI argv frontend MUST support help generation from command context, standard parameter metadata, and owner native option metadata without becoming the semantic owner of those parameters.

#### Scenario: Help includes owner metadata
- **WHEN** a direct CLI help invocation targets a command or adapter operation with registered standard parameters or native options
- **THEN** help output includes the applicable usage, defaults, possible values, and native option descriptions from metadata
- **THEN** help generation does not read config or execute the adapter operation
