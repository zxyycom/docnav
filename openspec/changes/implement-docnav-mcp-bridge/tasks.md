一句话核心：实现格式无关 MCP bridge，让 MCP Client 通过核心 `docnav` CLI 阅读文档。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现；未完成本项前，1.x 及后续任务全部处于阻塞状态。

## 1. MCP 包与 Tool 声明

- [ ] 1.1 （阻塞：等待 0.1 用户审计确认）建立 `docnav-mcp` Node.js/JavaScript 包和可安装 bin。
- [ ] 1.2 （阻塞：等待 0.1 用户审计确认）实现 stdio MCP transport 启动入口。
- [ ] 1.3 （阻塞：等待 0.1 用户审计确认）声明 `document_outline`、`document_read`、`document_find` 和 `document_info`。
- [ ] 1.4 （阻塞：等待 0.1 用户审计确认）内联或随包打包每个 tool 的 readable outputSchema。

## 2. CLI 映射

- [ ] 2.1 （阻塞：等待 0.1 用户审计确认）将 `document_outline` 参数映射到 `docnav outline`。
- [ ] 2.2 （阻塞：等待 0.1 用户审计确认）将 `document_read` 参数映射到 `docnav read`，ref 原样传递。
- [ ] 2.3 （阻塞：等待 0.1 用户审计确认）将 `document_find` 参数映射到 `docnav find`。
- [ ] 2.4 （阻塞：等待 0.1 用户审计确认）将 `document_info` 参数映射到 `docnav info`。
- [ ] 2.5 （阻塞：等待 0.1 用户审计确认）将 MCP `format` 原样映射为 `docnav --format`。

## 3. MCP 输出

- [ ] 3.1 （阻塞：等待 0.1 用户审计确认）将 `docnav` readable 结果转换为 MCP TextContent。
- [ ] 3.2 （阻塞：等待 0.1 用户审计确认）将 `docnav` readable 结果转换为 structuredContent，并通过 readable schema 校验。
- [ ] 3.3 （阻塞：等待 0.1 用户审计确认）确保 structuredContent 不包含 protocol envelope 字段。
- [ ] 3.4 （阻塞：等待 0.1 用户审计确认）实现 readable 错误到 MCP 错误输出的映射，保留必要 code/details。

## 4. 验证与审计

- [ ] 4.1 （阻塞：等待 0.1 用户审计确认）覆盖四个 MCP tools 的参数映射测试。
- [ ] 4.2 （阻塞：等待 0.1 用户审计确认）覆盖 TextContent、structuredContent 和 outputSchema 测试。
- [ ] 4.3 （阻塞：等待 0.1 用户审计确认）端到端验证 MCP 调用经由核心 `docnav` CLI，而不是直接调用 adapter。
- [ ] 4.4 （阻塞：等待 0.1 用户审计确认）用局部 diff 确认只修改 MCP bridge 和相关测试范围。
