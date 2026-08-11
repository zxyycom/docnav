# Tasks

本清单保留 MCP bridge 的完整交付顺序；`1.1` 是产品恢复、核心契约稳定性与 Current 重新基线的硬门禁，关闭前不得执行 `1.2` 至 `1.7` 或创建 package/production surface。

## Readiness

- [x] 0.1 确认 proposal、design 和 tasks 都以格式无关 stdio bridge 为同一交付目标，并保留产品延后事实。
- [x] 0.2 确认 bridge 只调用核心 CLI 的 protocol-json 路径，不复制 parser、routing、adapter selection、config 或 readable-view owner。
- [x] 0.3 确认四个 tools、两种 MCP content、failure、schema、package 和端到端证明都有对应任务。

## Implementation

- [ ] 1.1 取得明确的产品恢复确认，并从届时 adapter lifecycle、find、protocol、output、CLI 和 release owner 重新基线；同步本计划后再继续。
- [ ] 1.2 按项目测试流程恢复届时完整当前树与 Case 基线，确认本 design 是实施期间唯一承载 change-local Target 的载体，并登记 MCP bridge、CLI/protocol/output/schema/examples/testing/release 的预期 owner delta；此时不把 Target 写成 Current。
- [ ] 1.3 选择当前受支持的 MCP SDK 和最小 Node.js package 结构，新增可安装 `docnav-mcp` bin、stdio transport 与离线 package metadata。
- [ ] 1.4 声明 `document_outline`、`document_read`、`document_find`、`document_info` 的 input/output schemas，并建立从稳定项目 schema 生成或双向 fixture 校验的路径；覆盖届时 Current optional result branch。
- [ ] 1.5 实现四个 tools 到 `docnav <operation> ... --output protocol-json` 的 argv 映射，保持 ref 和 adapter id 原样传递，并证明 info 不接收分页参数。
- [ ] 1.6 实现进程结果与 protocol response 校验，从同一 normalized facts 生成 structuredContent 和 TextContent，覆盖成功 stderr 与稳定 failure 映射。
- [ ] 1.7 完成 package-local 安装/使用说明、workspace package 入口和必要的 release artifact 集成，作为可验证交付；不在本任务中新增或同步稳定 owner 为 Current。

## Verification

- [ ] 2.1 覆盖四个 tools 的 argv、input validation、output schema、success/failure content、malformed stdout 和成功 stderr tests。
- [ ] 2.2 在离线 package fixture 中启动 stdio server，证明 tool discovery 不依赖远程 schema 或直接 adapter access。
- [ ] 2.3 运行真实 `docnav` 端到端测试，证明 MCP facts 与 CLI protocol-json 一致，再运行 package 与 `bun run verify:docnav-workspace`，形成 implementation/behavior evidence。
- [ ] 2.4 在 `2.1` 至 `2.3` 的实现和行为证据通过后，新增 MCP bridge/package 稳定 owner，并把 design 登记且已成立的 CLI/protocol/output/schema/examples/testing/release delta 同步为 Current。
- [ ] 2.5 对同步后的 design、稳定 owner、package、tool schemas 和端到端证据做最终一致性验证；重新运行 docs/schema/example、package 与 `bun run verify:docnav-workspace`，再审查局部 diff，确认没有复制格式业务、完整 protocol envelope、第二次 CLI 调用、默认 readable parser 或未批准的 service runtime。
