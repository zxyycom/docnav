## 一句话核心

先审计这份临时 change，再实现 fast outline：小文档直接读，大文档回退 outline。

## 文档状态

当前 tasks 是未审计的临时执行清单。第一个任务必须先审计 proposal、design、spec 和 tasks；审计完成前，不要被本文中的方案顺序、字段名或命令名误导为最终实现指令。

## 1. 审计与定稿

- [ ] 1.1 审计 `add-fast-outline` 的 proposal、design、spec 和 tasks，确认命令名、字段名、输出形状、MCP tool 名称、协议边界和 adapter 责任边界是否成立。
- [ ] 1.2 根据审计结果修订 change artifacts，并把未决问题收敛为明确决策或保留为实现前阻塞项。
- [ ] 1.3 运行 OpenSpec 状态检查，确认 proposal、design、spec 和 tasks 仍完整可用。

## 2. 协议与 Schema

- [ ] 2.1 为协议 `OutlineResult` 类型和校验添加可选全文 ref 元数据。
- [ ] 2.2 更新 protocol response schema 和 readable outline/common schema，使需要的位置允许可选全文 ref。
- [ ] 2.3 添加 readable fast-outline schema，使用 `mode` 区分 read-mode 和 outline-mode。
- [ ] 2.4 更新验证 outline 结果形状的 protocol/readable 示例或 fixtures。

## 3. Markdown Adapter

- [ ] 3.1 让 Markdown adapter 在 outline 结果中填充 adapter 自有的全文 ref 元数据。
- [ ] 3.2 添加 adapter 测试，验证 outline 仍返回正常 entries，并额外暴露全文 ref 元数据。
- [ ] 3.3 添加 read 测试，验证暴露的全文 ref 可以原样传给 Markdown read。

## 4. 核心 CLI

- [ ] 4.1 添加 `docnav fast-outline <path>` 参数解析，支持 adapter、page、limit chars 和 output mode。
- [ ] 4.2 复用现有项目解析、path 检查、adapter 选择、默认值合并、warning 处理和错误映射。
- [ ] 4.3 实现 fast outline evaluation：先调用 outline，再用 adapter 提供的全文 ref 尝试受限 read；read `page: null` 时返回 read-mode，否则返回 outline-mode。
- [ ] 4.4 实现 fast-outline 两种 mode 的默认文本和 readable-json 输出。
- [ ] 4.5 根据审计后的决策实现 `fast-outline --output protocol-json` 行为。

## 5. MCP Bridge

- [ ] 5.1 添加专用 `document_fast_outline` MCP tool 声明，并内联输出 schema。
- [ ] 5.2 将 MCP tool call 映射到 `docnav fast-outline`，不解析文档内容或 ref 语法。
- [ ] 5.3 将 read-mode 和 outline-mode readable 结果转换为 MCP TextContent 和 structuredContent。

## 6. 文档与验证

- [ ] 6.1 如新命令改变 CLI 或 MCP 阅读路径，更新 `docs/navigation.md` 的角色阅读指引。
- [ ] 6.2 更新 `docs/cli.md`，说明 fast-outline 命令、输出形状和 MCP tool 映射。
- [ ] 6.3 更新 `docs/protocol.md` 和 `docs/adapter-contract.md`，说明可选 outline 全文 ref 元数据。
- [ ] 6.4 添加聚焦 CLI 测试，覆盖小文档 direct read、大文档 outline fallback、缺少 full-ref fallback 和现有 outline 行为不变。
- [ ] 6.5 添加 MCP bridge 测试，覆盖 `document_fast_outline` 映射和 structuredContent mode 选择。
- [ ] 6.6 运行目标 Rust/Node 测试、schema validation，并最终运行 `pnpm run verify:docnav-workspace`。
