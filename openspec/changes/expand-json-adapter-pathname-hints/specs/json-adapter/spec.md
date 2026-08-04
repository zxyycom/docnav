本临时 delta 计划在 `add-jsonc-comment-aware-navigation` 成为 Current 后，从 then-Current 主 spec 重建 JSON adapter 注册 requirement，完整保留其一个 `json` identity 与两个 descriptor content types，并仅扩展强 JSON-family pathname hint allowlist 与可验证 generic-routing 结果。

## MODIFIED Requirements

### Requirement: JSON adapter 必须作为静态 linked adapter 提供

`docnav-json` MUST 以 adapter id `docnav-json` 和一个 normalized format id `json` descriptor 暴露一个 registry-facing `AdapterDefinition` factory，并由 core static registry 链接进同一个 `docnav` 可执行文件。该 descriptor 的 `extensions[]` basename suffixes MUST 精确等于以下有序集合：`.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`；`filenames[]` exact basename hints MUST 精确等于以下有序集合：`.prettierrc`、`.watchmanconfig`、`Pipfile.lock`、`deno.lock`；`content_types[]` MUST 精确等于以下有序集合：`application/json`、`application/jsonc`。Pathname hints 与 declared content types MUST 只作为 format metadata；它们 MUST NOT 断言 document validity、选择 grammar/dialect/profile，或增加另一个 adapter/format identity。JSON adapter 的 executable set MUST 精确等于 package core `docnav` 单元素集合。它 MUST 实现固定的 outline、read、find 和 info strategy interface without a routing probe。其 public input surface MUST 等于既有 closed standard operation input；core parameter catalog、`StandardInputBinding`、CLI、env、config 和 protocol input inventory MUST 保持注册前的契约。JSON-specific 安全上限 MUST 由 adapter-private 单一硬编码配置源拥有。

#### Scenario: Core 检查内置 JSON adapter

- **WHEN** 用户运行 `docnav adapter list`
- **THEN** 结果包含 implementation source 为 `core_static` 的 `docnav-json`
- **THEN** manifest 声明一个 JSON format、`.json`、`.code-workspace`、`.jsonc`、`.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb` 与 `.sarif` `extensions[]` basename suffixes，`.prettierrc`、`.watchmanconfig`、`Pipfile.lock` 与 `deno.lock` exact filenames，以及 `application/json` 与 `application/jsonc`
- **THEN** listing 不执行 JSON selection probe
- **THEN** 发布包中的 core `docnav` executable 是 JSON operation 的交付入口

#### Scenario: JSON 使用既有公共输入

- **WHEN** navigation 选择 `docnav-json` 执行 document operation
- **THEN** adapter 接收对应 operation 的 closed standard input
- **THEN** strategy-visible 字段集合等于该 operation 的 common binding
- **THEN** matched filename、suffix、content type 或 format identity 不进入 strategy input
- **THEN** core public input inventory 与注册前相同

#### Scenario: JSON-family pathname hints 只启用 generic navigation

- **WHEN** complete basename 匹配 `.code-snippets`、`.jsonld`、`.geojson`、`.har`、`.webmanifest`、`.ipynb`、`.sarif`、`Pipfile.lock` 或 `deno.lock` 的 manifest hint
- **THEN** automatic routing 选择既有 `docnav-json` definition
- **THEN** selected operation 按 JSON adapter owner contract 处理实际文档，并复用其 grammar、generic structural navigation、ref、output 和 diagnostic 语义
- **THEN** hint 命中不引入 pathname-specific profile operation contract 或 public input
- **THEN** descriptor 精确声明 `application/json` 与 `application/jsonc`，但 pathname hint 或 declared content type 都不选择 grammar、dialect 或 profile
