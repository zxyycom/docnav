# Docnav

Docnav 是 CLI-first 的结构化文档导航系统。它用有限、可继续的读取流程，让人类、脚本、自动化、skill、项目提示词和 MCP 在不展开整份大文档的情况下定位内容。

核心产品路径是：

```text
outline -> ref -> read
```

`outline` 返回紧凑条目和 adapter 生成的 `ref`；调用方把 `ref` 原样传给 `read`，读取当前文档内部的对应区域。分页通过 `limit_chars` 和响应中的下一页 `page` 继续。

## v0 范围

v0 首期聚焦 Markdown 纵向链路：

- 核心入口设计为 `docnav` CLI，负责格式识别、adapter 路由、配置、项目初始化、输出模式和错误映射。文档操作默认使用 `readable-view` 输出（pretty JSON header + block section）。
- 当前仓库落地 `docnav-markdown` adapter、共享协议 crate、schema、示例和验证脚本。
- Markdown adapter 支持 `outline`、`read`、`find`、`info`、`manifest`、`probe` 和 `invoke`。
- JSON、YAML、TOML 和 INI 是后续 adapter 能力，不属于首期落地范围。

## Quick Start

准备 Node.js、pnpm 和 Rust toolchain 后安装依赖：

```bash
pnpm install
```

运行 Markdown adapter development smoke：

```bash
pnpm run smoke:docnav-markdown:dev
```

本地最小用法：

```bash
cargo run -p docnav-markdown -- outline crates/docnav-markdown/tests/fixtures/cli-smoke/normal.md
cargo run -p docnav-markdown -- read crates/docnav-markdown/tests/fixtures/cli-smoke/normal.md --ref "H:L1:H1:I1"
```

默认输出是 `readable-view`，适合直接阅读。需要结构化阅读结果时显式使用 `--output readable-json`；需要稳定机器协议时使用 `--output protocol-json` 或 adapter `invoke`。

发布制品生成、验证和 smoke 见 [测试策略](docs/testing.md)。

## 验证

轻量文档链接检查：

```bash
pnpm run validate:links
```

文档、schema、示例和语义校验：

```bash
pnpm run validate:docs
```

日常快速验证入口：

```bash
pnpm run verify:docnav-workspace:required
```

交付前综合验证入口：

```bash
pnpm run verify:docnav-workspace
```

## 文档入口

- [文档导航](docs/navigation.md)：角色化阅读路径、文档分层、规则 owner 和术语。
- [架构](docs/architecture.md)：制品职责、接入方式、语义层、配置所有权和进程边界。
- [CLI 与 MCP 输出](docs/cli.md)：命令、输出模式和 MCP 映射。
- [原始协议](docs/protocol.md)：invoke envelope、operation、page 和稳定错误。
- [适配器契约](docs/adapter-contract.md)：adapter 命令、manifest、probe 和 invoke 行为。
- [Ref](docs/refs.md)：ref 的共享调用流程、非空 opaque string 和 adapter 所有权。
- [测试策略](docs/testing.md)：自动化测试层级、验收矩阵和一致性审计。
