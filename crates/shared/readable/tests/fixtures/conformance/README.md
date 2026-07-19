# Conformance vector fixtures

This directory contains committed, version-controlled JSON files that serve as
cross-implementation acceptance material for the readable-view renderer.
The set is intentionally representative rather than exhaustive: renderer Rust
unit tests cover finer-grained internal edge cases, while these fixtures define
portable semantics for other implementations.

Each file describes an input JSON value, view kind, optional config override,
optional expected failure, and a list of order-independent assertions.

See `crates/shared/readable/README.md` for the full conformance contract,
assertion types, and the explicitly excluded stability surface.

## Covered scenarios

| File | Scenario |
|---|---|
| `01_no_block_outline.json` | No blocks (outline) |
| `04_single_block.json` | Single block (read) |
| `07_chinese.json` | Chinese characters (UTF-8 byte length) |
| `10_crlf_payload.json` | CRLF payload preservation |
| `11_no_trailing_newline.json` | Framing LF for payload without trailing LF |
| `12_block_marker_in_body.json` | Block marker text in payload body |
| `14_readable_error.json` | Readable error primary diagnostic fields |
| `15_error_guidance_array.json` | Error guidance array and diagnostic context in header |
| `16_undeclared_extension_fields.json` | Undeclared extension fields |
| `17_order_independent_assertions.json` | Order-independent assertions |
| `18_renderer_failure_missing_pointer.json` | Renderer failure — missing pointer |
| `19_renderer_failure_non_string.json` | Renderer failure — non-string target |
| `20_outline_unstructured_content_block.json` | Unstructured outline `/content` block |
| `21_outline_auto_read_nested_content_block.json` | Structured outline auto-read `/auto_read/read/content` block |
