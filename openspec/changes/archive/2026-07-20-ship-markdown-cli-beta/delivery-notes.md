# Docnav v0.1.0-beta.1 Delivery Notes

## Release Identity

| Field | Evidence |
| --- | --- |
| Commit | `c964f27e8f40780e367ec53945ab133160000ed6` |
| Annotated tag | `v0.1.0-beta.1` (`840f2b8b831e5c5fc0deb805c67e648abf88bbb5`) |
| Release | <https://github.com/zxyycom/docnav/releases/tag/v0.1.0-beta.1> |

## Workflow Evidence

| Run | Result |
| --- | --- |
| [Non-publish rehearsal 29722275359](https://github.com/zxyycom/docnav/actions/runs/29722275359) | PASS |
| [Publish 29723248946](https://github.com/zxyycom/docnav/actions/runs/29723248946) | PASS |

## Published Assets

| Target | Binary | Binary SHA-256 | Checksum asset SHA-256 | Canonical manifest SHA-256 |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `docnav-v0.1.0-beta.1-x86_64-unknown-linux-gnu` | `99983e1bb97d53e3fa85564484a2e9e9efe03ff3f3b858a2aee13c1b9caeceb9` | `cab58548b28f5249c9be579f26d9e34c43fff4c88a7e834c5771bbf25da876b7` | `0198231234d1dd04270a46c0ad5ee89cec6cd1c30dffa6b4ace04a224d460b6a` |
| `x86_64-pc-windows-msvc` | `docnav-v0.1.0-beta.1-x86_64-pc-windows-msvc.exe` | `3a2fce61e0ad8b94bcd23d37fb9c0a7ff85dc4dfe962a10d9420e675d9558ca4` | `50e4b4810462013d8d06c6b534f2a7e900e52e01454d6300de386bd0089eccfd` | `e0795a780637c69cfb98dd94f3c0164ec1a0d72cc67b1605cb257ca52a5a42fd` |

The release contains exactly these two binaries and their two matching `.sha256` files. Exact-set and checksum verification passed.

## Execution Evidence

- Linux direct execution passed for `version`, `--help`, `info`, `outline`, and `read`; the version output was `docnav 0.1.0-beta.1`.
- The downloaded Windows executable was byte-identical to the same-run public and canonical package executable. Native [Windows job 88290541889](https://github.com/zxyycom/docnav/actions/runs/29723248946/job/88290541889) passed its 50-command smoke suite, and the executable embeds `docnav 0.1.0-beta.1`.
- The Windows result is equivalent execution evidence from the native Windows job; it does not claim that the PE executable was directly executed on Linux.

## Final Status

**PASS** — release identity, exact public asset set, checksums, binary version, and basic execution evidence all passed.
