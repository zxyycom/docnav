# docnav-typed-fields-macros

Proc-macro implementation for canonical
[`docnav-typed-fields`](../typed-fields/README.md) declarations. It provides the `FieldDefs` derive
used to generate a definition set and typed materialization path from one field declaration.

Most consumers use the derive through `docnav_typed_fields::FieldDefs` rather than depending on
this implementation package directly. Source extraction, priority, merge execution, and framework
integration belong to the other packages in Docnav's root Rust workspace.
