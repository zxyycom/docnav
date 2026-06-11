本 change 的目标是明确 adapter-owned ref 的共享契约边界，并在 Markdown adapter 中采用不包含标题内容的结构 ref。当前提案已通过设计审计并解除实施门禁，尚未应用到现行主规范或实现。

## Why

当前 Markdown heading ref 将完整 breadcrumb 编入 ref，长度随标题文本、层级深度和字符集增长，增加 CLI 传递、复制、转义和分页预算成本。

现行共享文档还把唯一定位、消歧、文档变化后的失败方式等行为写成所有 adapter 的共同保证。这超出了共享层需要拥有的边界，也限制了后续格式 adapter 按自身结构选择 ref 语义。

## What Changes

- **BREAKING**：Markdown heading ref 改为 `H:L{line}:H{level}:I{index}`。该格式不包含 title、breadcrumb 或内容摘要，长度只随结构数字的位数增长。
- Markdown adapter 将 `line`、heading `level` 和全文 heading `index` 作为当前解析结果中的结构坐标，并用字段前缀保留基础可读性和人工审计能力。
- Markdown outline 将标题或 breadcrumb 导航语义放入 `display`；find 保留匹配位置附近的文本片段，并可补充 heading 导航语义。超长 display 可以按字符预算截断，ref 始终保持完整。
- Markdown adapter 在当前解析结果中精确匹配三个字段。文档内容或 parser 结果变化后，旧 ref 不保证继续表示同一 heading。
- 新增稳定错误 `REF_INVALID`。Markdown ref 不符合 adapter 当前 grammar 时返回 `REF_INVALID`；格式合法但当前解析结果中没有匹配项时返回 `REF_NOT_FOUND`。
- 旧 Markdown heading ref 不形成专属兼容语义，只作为不符合当前 grammar 的输入示例。
- Markdown adapter 保留私有 ref `doc:full`，用于读取整篇 Markdown 文档。
- 共享 ref 契约保留强制调用流程：adapter 在 outline 或 find 中生成 ref，调用方将 path 和 ref 原样提交给 read，core 按 path 选择 adapter 并原样传递 ref，adapter 返回读取结果或稳定错误。
- 共享传输边界只要求 ref 是 adapter 生成和解释的非空 opaque string；共享协议、`docnav` 和接入层只校验基本字段形状并原样传递，不承诺 adapter 的解释结果。
- 唯一性、稳定性、消歧、定位粒度、多个 ref 是否可指向同一区域、文档变化后的行为，以及 adapter 是否接受某个 ref，均由对应 adapter 自行定义。
- 新增 `docs/adapters/markdown.md`，集中记录 Markdown 的 outline、read、find、ref、全文读取、保证范围、错误分类和验证入口。

## Non-Goals

- 不为所有 adapter 规定统一 ref grammar、唯一性、身份稳定性或消歧策略。
- 不在 core 或 MCP 中解析、转换、推断或兼容 Markdown ref。
- 不为 Markdown ref 增加文档版本、mtime、内容 hash 或过期检测。
- 不保证 Markdown ref 跨文档修改或 parser 版本变化继续指向同一 heading。
- 不为旧 Markdown heading ref 提供兼容读取期或双 grammar 迁移。

## Confirmed Decisions and Review Boundary

以下决策已经确认，并作为本 change 的评审边界：

1. `outline/find -> ref -> read` 是强制且稳定的共享调用流程。
2. core、协议和 MCP 只校验共享字段 shape，并将非空 opaque ref 原样传给按 path 选定的 adapter。
3. 共享层不保证 `read` 必然成功、唯一定位或返回特定区域。ref 的 grammar、适用 operation、有效条件、唯一性、消歧、读取结果和错误分类由 adapter 专属契约定义。
4. 该边界是正确性责任的分层，不是为了缩短 ref 而放弃正确性：共享层负责 adapter 选择、原样传递和稳定错误映射；adapter 负责其生成、解释、定位和错误行为符合自身契约。
5. Markdown heading ref 是结构快照，不是内容身份。文档变化后旧 ref 可能失效、继续匹配或匹配其它当前 heading，这是 Markdown adapter 明确接受的行为。
6. Markdown outline display 承载标题或 breadcrumb；find display 保留匹配位置附近的文本片段，并可补充 heading 导航语义。超长 display 可以截断。
7. 新 grammar 生效后旧 Markdown heading ref 直接进入 `REF_INVALID`，不提供兼容读取或双 grammar 迁移。

后续审查应检查 artifacts、实现、测试和错误映射是否落实这些决策。不同设计偏好、通用 API 惯例、旧 ref 兼容偏好、内容身份偏好或要求共享层提供更强定位保证，不构成重新讨论这些决策的理由。

只有出现以下情况时才重新评估对应决策：与现行主规范存在可定位的实质冲突；存在明确不可实现条件；出现可复现的契约缺陷；或用户明确要求修改该决策。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-adapter-v0-implementation`：修改 Markdown heading ref 的格式、生成、读取、保证范围和错误分类。
- `v0-contract-documentation`：将共享 ref 契约收敛为非空 opaque string 与原样传递，并增加 `REF_INVALID` 稳定错误边界和 adapter 专属文档所有权。

## Impact

- 修改 `docnav-markdown` 的 heading ref 生成与解析，以及 outline、find、read 的可观察输出。
- 修改 Markdown outline/find 的 display，使其在 ref 不再包含标题后继续提供可截断的导航语义。
- 扩展稳定错误集合，增加 `REF_INVALID` 及其 `ref`、`reason` details，并同步 protocol/readable schema、错误规则、示例和各输出层映射。
- 更新包含 Markdown heading ref 的规范、文档、示例、fixture、golden output 和测试。
- 删除共享层对 ref 唯一性、消歧、身份稳定性和读取成功的保证，但继续保持 adapter 所有权与 core/MCP 原样传递边界。
- 不改变分页模型、core adapter routing 或 MCP bridge 的格式无关职责。
