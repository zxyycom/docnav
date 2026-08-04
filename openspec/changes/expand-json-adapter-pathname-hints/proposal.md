本临时 change 计划在 JSONC predecessor 成为 Current 后，将一组强 JSON-family pathname hints 加入既有 `docnav-json` manifest，完整保留其两个 descriptor content types，并保持 generic structural navigation 与现有 routing、grammar owner 边界。

## Why

若 pathname 不以 `.json` 结尾，多个常见且结构上属于 JSON family 的文档目前无法通过 automatic pathname routing 到达 `docnav-json`。在不引入 profile-specific 行为的前提下扩展高置信度 hints，可以让这些文档复用既有 `outline -> ref -> read` 能力；将该工作排在 `add-jsonc-comment-aware-navigation` 之后，则避免 `.code-snippets` 的常见 JSONC 内容和两个并行 delta 对同一注册 requirement 形成不确定基线。

## What Changes

- 在 predecessor `add-jsonc-comment-aware-navigation` 已成为 Current 后，计划从 then-Current `json-adapter` 注册 requirement 完整重建一个 `MODIFIED` requirement，保留其一个 `json` identity、`.json` / `.code-workspace` / `.jsonc` suffixes、`.prettierrc` / `.watchmanconfig` exact filenames，以及 descriptor content types `application/json` / `application/jsonc`，再加入 normalized suffixes `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif` 和 exact filenames `Pipfile.lock`、`deno.lock`。
- 所有新增项只作为 manifest pathname selection hints；被选中的文档继续由 then-Current `docnav-json` grammar、generic structural navigation、ref、read、find、info、full-read、diagnostic 和 no-fallback 契约处理。Hint 命中与 predecessor 声明的两个 content types 都不证明内容有效，也不选择 profile 或 grammar mode。
- 同步 JSON adapter owner 文档、OpenSpec 主 spec、manifest inspection/listing evidence、语义 Case、目标测试、coverage 与 release-package smoke；不改变或解释 predecessor 的 descriptor/result content-type 语义，也不改变 shared protocol、public input、format id `json` 或 adapter id `docnav-json`。
- 将 strict-JSON-compatible hints 的语义与 JSONC grammar change 分离：本 change 不拥有 JSONC grammar，但为避免 `.code-snippets` 与注册 requirement 的叠加冲突，整体实施仍以 predecessor Current 为前置条件。

## Non-Goals

- 不验证 JSON-LD、GeoJSON、HAR、Web App Manifest、Jupyter Notebook、SARIF、Pipfile lock 或 Deno lock profile，也不提供 profile-specific outline、ref、read、find、metadata、schema、语义检查或远程 context/resource resolution。
- 不新增 JSON5、NDJSON、JSONL、JSON Text Sequences、含义不明确的 rc filename、弱 generic basename、binary JSON-like format 或其它未列出的 pathname hint。
- 不为已经由 `.json` suffix 覆盖的 `package.json`、`tsconfig*.json`、`deno.json` 等名称增加重复 exact-filename hints。
- 不改变 Current owner contract 定义的 routing algorithm、route-before-document-I/O、explicit selection、selected-operation parse 或 selected failure no-fallback 行为，也不新增、删除、选择或重新解释 descriptor/result content type。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `json-adapter`: 在 predecessor 成为 Current 后修改既有 registry-facing manifest requirement 的完整 pathname hint allowlist，保留其两个 descriptor content types，并要求相应跨层验证证据。

## Impact

预期实施面限于 built-in `docnav-json` 的 static manifest allowlist，以及拥有或证明该 observable listing/routing 行为与 predecessor descriptor content-type set 保持不变的 JSON adapter 文档、主 spec、Case ledger、目标测试、coverage mapping、core CLI/release smoke 和相关示例材料。不会新增依赖、adapter、format identity、content type、protocol/schema field、CLI option、typed field、ref grammar、continuation shape 或 wrapper behavior；本 artifact 仅形成待审计计划，不证明实现已获批准或已成为 Current。
