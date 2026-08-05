本 change 扩展 `docnav-json` 的 closed pathname-hint allowlist，使九类高置信度 JSON-family 文件能够通过 automatic pathname routing 进入既有 generic JSON navigation。它不增加 profile-specific 行为，也不改变 JSONC-capable grammar、content type 或公共协议。

## Why

Current automatic routing 只识别 `.json`、`.code-workspace`、`.jsonc`、`.prettierrc` 和 `.watchmanconfig`。因此 `.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`、`.code-snippets`、`Pipfile.lock` 与 `deno.lock` 即使包含当前 `docnav-json` 可以解析的结构，也需要调用者显式选择 adapter。

这些 pathname 对 JSON-family 内容具有足够强的格式提示。把它们加入 manifest 可以复用现有 `outline -> ref -> read` 流程，同时保持“pathname 负责选择、selected adapter 负责验证实际内容”的边界。

## Observable Result

- `docnav adapter list` 显示扩展后的完整、有序 JSON pathname allowlist。
- 对九个新增 pathname 执行未指定 adapter 的 document operation 时，automatic routing 选择既有 `docnav-json`。
- 解析成功时，用户获得与 `.json` / `.jsonc` 相同的 generic structural navigation、opaque JSON ref、read、find、info、full-read、pagination 和 output 行为。
- 内容不符合当前 JSON adapter grammar 时，调用返回 JSON-owned parse diagnostic；不会因 hint 命中而宣称 profile 有效，也不会重新 routing 或 fallback。

## What Changes

修改既有 `json-adapter` 注册 requirement 的完整 pathname allowlist：

- `extensions[]` 在 Current `.json`、`.code-workspace`、`.jsonc` 之后，依次追加 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`。
- `filenames[]` 在 Current `.prettierrc`、`.watchmanconfig` 之后，依次追加 `Pipfile.lock`、`deno.lock`。
- `content_types[]` 保持 `application/json`、`application/jsonc`。
- adapter id `docnav-json`、format id `json`、一个统一 JSONC-capable grammar、closed standard input 与 shared protocol/output shape 均保持不变。

## Non-Goals

- 不验证 JSON-LD、GeoJSON、HAR、Web App Manifest、Jupyter Notebook、SARIF 或 lockfile profile，不增加 schema、domain semantics 或远程 resource resolution。
- 不新增 JSON5、NDJSON、JSONL、JSON Text Sequences、模糊 rc filename、弱 generic basename、binary JSON-like format 或其它未列出的 hint。
- 不为已由 `.json` suffix 覆盖的 `package.json`、`tsconfig*.json`、`deno.json` 等名称增加重复 exact-filename entries。
- 不改变 exact-filename/suffix lookup、normalization、precedence、route-before-document-I/O、explicit selection 或 selected-failure no-fallback 行为。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `json-adapter`: 扩展 registry-facing manifest requirement 的 exact pathname-hint allowlist，并要求 listing、selection、navigation 与 release evidence 对相同集合闭合。

## Impact

预期 production diff 只修改 built-in `docnav-json` manifest 的 `extensions[]` 与 `filenames[]` 数据。同步面包括 JSON owner 文档、OpenSpec 主 spec、semantic Cases、manifest/registry assertions、automatic-selection tests、CLI smoke 和 release-package evidence。不会新增 dependency、CLI option、typed field、protocol/schema shape、ref grammar、continuation shape 或 output mode。
