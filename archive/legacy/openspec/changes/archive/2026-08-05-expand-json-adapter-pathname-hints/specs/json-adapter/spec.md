本 delta 完整修改 Current `json-adapter` 注册 requirement：保留一个 `docnav-json` / `json` identity、统一 JSONC-capable grammar、两个 descriptor content types 与 closed public input，只扩展 pathname-hint allowlist 和对应 generic-routing 结果。

## MODIFIED Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供

`docnav-json` MUST 以 adapter id `docnav-json` 和一个 normalized format id `json` descriptor 暴露 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。该 descriptor 的 `extensions[]` basename suffixes MUST 精确等于 `.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb` 和 `.sarif`；`filenames[]` exact basename hints MUST 精确等于 `.prettierrc`、`.watchmanconfig`、`Pipfile.lock` 和 `deno.lock`；`content_types[]` MUST 精确等于 `application/json` 与 `application/jsonc`。它 MUST NOT 声明其它 JSON-family pathname hint、adapter identity 或 format identity。

JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 outline、read、find 和 info strategy interface without a routing probe。其 public input surface MUST 等于既有 closed standard operation input；注册 JSONC 与本 requirement 的 pathname hints MUST NOT 增加 core parameter、`StandardInputBinding`、CLI、env、config 或 protocol input。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明一个 JSON format、`.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb` 与 `.sarif` suffixes，`.prettierrc`、`.watchmanconfig`、`Pipfile.lock` 与 `deno.lock` exact filenames，以及 `application/json` 与 `application/jsonc`
- **THEN** listing 不执行 JSON selection probe
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用既有公共输入

- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** matched filename、suffix、content type 或 format identity 不进入 strategy input
- **THEN** core public input inventory 不包含 JSON-specific field

#### Scenario: 所有 JSON pathname 使用同一个 grammar

- **WHEN** descriptor pathname hint 或 explicit adapter intent 选择 `docnav-json`
- **THEN** selected operation 使用同一个 JSONC-capable grammar
- **THEN** pathname 和 descriptor content type 不选择 strict/JSONC dialect 或 pathname-specific profile

#### Scenario: 新增 JSON-family pathname hints 只启用 generic navigation

- **WHEN** complete basename 匹配 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`、`Pipfile.lock` 或 `deno.lock` 的 manifest hint
- **THEN** automatic routing 选择既有 `docnav-json` definition
- **THEN** selected operation 按 JSON adapter owner contract 处理实际文档，并复用其 generic structural navigation、ref、output 和 diagnostic 语义
- **THEN** hint 命中不证明 document 或 domain profile 有效
- **THEN** selected grammar failure 不重新 routing 或 fallback
