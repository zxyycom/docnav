# docnav-readable

Readable payload and readable-view rendering helpers for the Docnav document navigation system.

## Overview

This crate provides:

- **Single-path typed payload → `serde_json::Value`** — `readable-json` and `readable-view` derive from the same complete JSON value.
- **Repo-internal renderer config** — declares which JSON Pointer fields are block-eligible per view kind.
- **Readable-view renderer** — emits a pretty JSON header followed by length-delimited `[block …]` / `[endblock …]` sections with platform-independent LF framing.
- **Committed conformance vectors** — cross-implementation acceptance material for verifying renderer correctness.

The readable-view format is:

```text
<pretty JSON header>\n
\n
[block <json-pointer> bytes=<utf8-byte-length>]\n
<exact UTF-8 payload bytes>
[optional framing LF when payload has no trailing LF]
[endblock <json-pointer>]\n
```

When the renderer config declares no block fields, output is only the pretty JSON header plus its final LF byte. When blocks are present, the header field value is replaced with `{"$block": "<json-pointer>", "bytes": <utf8-byte-length>}` and a blank separator line appears between the header and the first block marker. Framing bytes are always LF (`0x0A`) on every platform; CRLF inside payload strings remains payload data and counts toward `bytes`.

## Conformance vectors

### Location

`tests/fixtures/conformance/*.json` — committed, version-controlled JSON files that serve as the portable source of truth for readable-view cross-implementation expectations. They are representative semantic vectors, not the full Rust renderer unit-test matrix.

### Cross-implementation goal: semantic consistency

Conformance vectors define a **semantic contract** for the readable-view format. Any implementation (Rust or future ports) that consumes the same vector files and passes the same assertions produces **semantically equivalent** output.

Rust unit tests in `src/renderer.rs` continue to cover fine-grained internal edge cases. The conformance set stays compact so non-Rust implementations can adopt it without inheriting duplicate CLI-display coverage.

### Stable assertion scope

Assertions focus on these stable semantics:

| Assertion target | What is guaranteed |
|---|---|
| **Block pointer** | The JSON Pointer (e.g. `/content`) that identifies a block field |
| **Byte length** | UTF-8 byte count of the block payload |
| **Block payload** | The extracted string content of a block field |
| **Header field** | A JSON field at a given Pointer has a specific value |
| **No blocks** | A view kind produces zero `[block]` sections |

### Explicitly excluded from the stable contract

The following are **NOT** part of the stable semantic contract and MUST NOT be asserted:

| Exclusion | Reason |
|---|---|
| **Header JSON key order** | `serde_json` and other implementations may serialize keys in different orders; structural equality via JSON Pointer lookups is the stable check |
| **Multi-block output order** | When a view declares multiple block fields, their `[block …]` sections may appear in any order; each block is checked independently by pointer |
| **Byte-for-byte consistency** | Output may differ in header whitespace or object member order while remaining semantically equivalent; framing LF bytes, block pointers, byte lengths and payload restoration remain stable |

### Adding a new vector

1. Create a JSON file under `tests/fixtures/conformance/` following the schema below.
2. Add a `#[test]` function in `tests/conformance_tests.rs` that loads the file via `load_vector!`.
3. Run `cargo test -p docnav-readable` to verify.

### Vector file schema

```jsonc
{
  "description": "Human-readable description of what this vector tests",
  "view_kind": "outline|read|find|info|error",
  "config_override": null,                // or { "blocks": ["/pointer1", …] }
  "expected_failure": null,               // or { "error_id": "…", "message_contains": "…" }
  "input": { /* JSON value to render */ },
  "assertions": [
    // One or more of:
    { "type": "block", "pointer": "/field", "byte_length": 42, "payload_contains": "text" },
    { "type": "no_blocks" },
    { "type": "header_field", "pointer": "/ref", "value": "L5" },
    { "type": "header_contains", "text": "\"$block\"" },
    { "type": "output_contains", "text": "[block /content" },
    { "type": "output_not_contains", "text": "[block" },
    { "type": "no_cr_in_framing" }
  ]
}
```

### Assertion types

| `type` | Semantics |
|---|---|
| `block` | A block section with the given `pointer` exists; optional `byte_length` and `payload_contains` checks |
| `no_blocks` | No `[block …]` / `[endblock …]` markers appear in the output |
| `header_field` | The header JSON value at `pointer` equals the expected `value` (structural comparison, order-independent for objects) |
| `header_contains` | The header JSON string contains the given text |
| `output_contains` | The full output string contains the given text |
| `output_not_contains` | The full output string does NOT contain the given text |
| `no_cr_in_framing` | The output contains zero CR (`\r`, 0x0D) bytes — framing uses LF only |

## Architecture boundary

This crate owns readable payload/value conversion, renderer config, `ReadableViewKind`, readable-view block framing, and conformance vectors. It does NOT own output mode dispatch, protocol envelopes, adapter selection, document parsing, or CLI wiring. Those responsibilities stay with their existing crates.

## Testing

```bash
# Run all tests including conformance vectors
cargo test -p docnav-readable

# Run only conformance vector tests
cargo test -p docnav-readable --test conformance_tests
```
