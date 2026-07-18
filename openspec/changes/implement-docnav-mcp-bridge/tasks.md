一句话核心：实现格式无关 MCP bridge，让 MCP Client 通过核心 `docnav` CLI 阅读文档。

## 0. 审计门禁

- [ ] 0.1 用户审计确认：用户已审计本 change 的 proposal、design、spec 和 tasks，并明确允许开始实现。

执行说明：0.1 完成前只进行审阅和文案修正；0.1 完成后按 1.x-4.x 执行实现与验证。

## 1. MCP 包与 Tool 声明

- [ ] 1.1 建立 `docnav-mcp` Node.js/JavaScript 包和可安装 bin。
  验收：包可通过 workspace 脚本构建，bin 名称为 `docnav-mcp`。
- [ ] 1.2 实现 stdio MCP transport 启动入口。
  验收：MCP Client 启动 `docnav-mcp` 后可读取 tool 列表。
- [ ] 1.3 声明 `document_outline`、`document_read`、`document_find` 和 `document_info`。
  验收：四个 tool 的输入参数覆盖 path、ref、query、page、limit_chars 和可选 adapter 中各 operation 需要的字段。
- [ ] 1.4 内联或随包打包每个 tool 的 MCP outputSchema。
  验收：tool 声明在离线环境中包含对应 outputSchema，不依赖远程 schema URL。

## 2. CLI 映射

- [ ] 2.1 将 `document_outline` 映射到核心 `docnav outline`。
  验收：path、page、limit_chars 和 adapter 按 CLI 参数传递。
- [ ] 2.2 将 `document_read` 映射到核心 `docnav read`。
  验收：path、ref、page、limit_chars 和 adapter 按 CLI 参数传递，ref 原样传递。
- [ ] 2.3 将 `document_find` 映射到核心 `docnav find`。
  验收：path、query、page、limit_chars 和 adapter 按 CLI 参数传递。
- [ ] 2.4 将 `document_info` 映射到核心 `docnav info`。
  验收：path 和 adapter 按 CLI 参数传递，page/limit_chars 不写入 info 调用。
- [ ] 2.5 固定所有 document tool 的输出模式。
  验收：每次调用核心 CLI 都传入 `--output protocol-json`，bridge 校验 stdout `ProtocolResponse` 后再映射 structuredContent。
- [ ] 2.6 保持 MCP adapter 参数为透传参数。
  验收：MCP 可选 `adapter` 原样映射为 `docnav --adapter <adapter-id>`；adapter id 解释、格式识别和候选继续遍历由核心 CLI 完成。

## 3. MCP 输出

- [ ] 3.1 将 `ProtocolResponse::Success.result` 转换为 MCP structuredContent。
  验收：structuredContent 通过对应 MCP outputSchema 校验，并只保留 operation result 中由该 tool 拥有的字段。
- [ ] 3.2 将同一 protocol result/error facts 渲染为 MCP TextContent。
  验收：TextContent 包含精简阅读文本，不解析默认 `readable-view`，不复制 Rust block framing，也不发起第二次 CLI 调用。
- [ ] 3.3 保持 structuredContent 的 MCP 边界。
  验收：structuredContent 不包含 `protocol_version`、`request_id`、`operation` 或 `ok`；read structuredContent 保留 `content_type`。
- [ ] 3.4 处理成功退出的 stderr status。
  验收：`docnav` 子进程退出码为 0 时，stderr 非空不升级为 MCP 错误；成功/失败判定以退出码和 stdout protocol response 的 `ok` 字段为准。
- [ ] 3.5 实现 protocol error 到 MCP 错误输出的映射。
  验收：MCP TextContent 和 structuredContent 映射 `ProtocolResponse::Failure.error` 的 code/message/owner，并在存在时保留 guidance/details，不暴露完整 protocol envelope。

## 4. 验证与审计

- [ ] 4.1 覆盖四个 MCP tools 的参数映射测试。
  验收：测试断言生成的 `docnav` argv 包含 operation 参数和 `--output protocol-json`。
- [ ] 4.2 覆盖 structuredContent、TextContent 和 outputSchema 测试。
  验收：每个 operation 的 successful structuredContent 可按 MCP outputSchema 校验，TextContent 渲染不改变 structuredContent shape；failure case 覆盖 protocol error mapping。
- [ ] 4.3 覆盖 stderr 成功 status 测试。
  验收：核心 CLI 成功退出且 stderr 非空时，MCP bridge 返回成功结果。
- [ ] 4.4 端到端验证 MCP 调用经由核心 `docnav` CLI。
  验收：测试确认 bridge 不直接调用 adapter，MCP structuredContent 与 CLI `protocol-json` result/error facts 一致。
- [ ] 4.5 完成局部审计。
  验收：用局部 diff 确认实现只修改 MCP bridge、相关测试和必要 workspace 包装入口。
