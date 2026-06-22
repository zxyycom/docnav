本 tasks 定义 adapter direct CLI 配置路径参数、配置读取、默认值合并与 native options 显式化的实现入口；它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“adapter direct CLI 通过 SDK 可覆盖配置路径读取自身配置，并显式化默认值/native options”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确复用 `adapter-protocol`、`docnav-contracts` 和 `markdown-navigation`，且没有创建一次性、同义或过宽 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/implement-adapter-direct-cli-config/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [ ] 1.4 审计 proposal、design 和 specs 是否明确 core `docnav`、MCP 和 adapter `invoke` 不读取 adapter direct CLI 配置。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、示例更新、测试更新或代码改动。

## 2. 规范与验证材料同步

- [ ] 2.1 按 `docs/navigation.md` 读取 `docs/CODING_STYLE.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/adapters/markdown.md`、`docs/testing.md` 和 `docs/testing/case-maintenance.md` 中与本 change 相关的 owner 规则。
- [ ] 2.2 更新 `docs/cli.md`，记录 adapter direct CLI JSON 配置默认路径、`--project-config-path` / `--user-config-path`、优先级、支持 key、错误输出形态和 `invoke` 不读配置。
- [ ] 2.3 更新 `docs/adapter-contract.md`，说明 adapter SDK direct CLI config path parameters、config helper、native options 显式化和 strict invoke transport 的边界。
- [ ] 2.4 更新 `docs/adapters/markdown.md`，记录 `docnav-markdown.json` 支持 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`。
- [ ] 2.5 更新测试策略和用例维护材料，记录 adapter direct CLI 配置优先级、非法配置和 invoke 不读配置的验证目标。

## 3. Adapter SDK 配置基础设施

- [ ] 3.1 在 `docnav-adapter-sdk` direct CLI 层新增配置路径参数模型，包含项目级配置路径和用户级配置路径的默认值，以及 `--project-config-path` / `--user-config-path` 覆盖值。
- [ ] 3.2 在 `docnav-adapter-sdk` direct CLI 层新增配置内容模型，包含 `defaults.limit_chars`、`defaults.output` 和 `options` object。
- [ ] 3.3 实现项目级配置默认路径 `.docnav/<adapter-id>.json` 与覆盖路径解析；缺失文件不得报错。
- [ ] 3.4 实现用户级 `<adapter-id>.json` 默认路径与覆盖路径解析，复用 core 当前用户配置目录规则或等价 helper；缺失文件不得报错。
- [ ] 3.5 实现配置合并优先级：显式 argv > 项目级配置 > 用户级配置 > 内置默认值，并只把配置贡献的 `limit_chars`、`output` 和适用 native options 写入标准 direct CLI operation 参数；`path`、`ref`、`query` 和 `page` 不从配置生成。
- [ ] 3.6 使用 `NativeOptionSpec` 校验 `options` key、value 和 operation 适用性，并只把当前 operation 适用的 native options 写入最终 operation input。
- [ ] 3.7 实现配置错误输出，覆盖 JSON parse failure、unknown key、invalid limit_chars、invalid output、invalid native option value 和配置路径不可读，并确认 `protocol-json` / `readable-json` / `readable-view` 的 stdout、stderr 和 exit code 符合输出模式契约。

## 4. Markdown Adapter 接入

- [ ] 4.1 更新 `docnav-markdown` direct CLI wiring，传入 adapter id、配置路径默认值、内置默认值和 native option specs 给 SDK config helper。
- [ ] 4.2 支持 `.docnav/docnav-markdown.json` 和用户级 `docnav-markdown.json` 的 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`，并确认 `options.max_heading_level` 对 outline 与 find 生效。
- [ ] 4.3 确认 `--limit-chars`、`--output` 和 `--max-heading-level` 显式 argv 覆盖配置文件值。
- [ ] 4.4 确认项目级/用户级配置路径覆盖值生效，且覆盖后的默认路径不参与本次合并。
- [ ] 4.5 确认 read/info 等不适用 `max_heading_level` 的 operation 不把该 option 写入 options。
- [ ] 4.6 确认 `docnav-markdown invoke`、manifest、probe 和 help 不读取 document operation 配置。

## 5. 测试与示例覆盖

- [ ] 5.1 增加 `docnav-adapter-sdk` 单元测试，覆盖 config path defaults/override、merge precedence、native option validation 和 missing file behavior。
- [ ] 5.2 增加 `docnav-markdown` CLI smoke fixture，覆盖项目级配置、用户级配置、配置路径覆盖、显式 argv 覆盖、内置默认 fallback，以及 outline/find 的 `options.max_heading_level` 行为。
- [ ] 5.3 增加负向矩阵测试，覆盖非法 JSON、未知 key、非法 `defaults.limit_chars`、非法 `defaults.output`、非法 `options.max_heading_level`，以及 `--output protocol-json` 下配置错误仍输出 protocol failure envelope。
- [ ] 5.4 增加 invoke 边界测试，证明 adapter `invoke` 不读取 adapter direct CLI 配置，也不补全缺失 protocol request fields。
- [ ] 5.5 如测试函数、case 归属或公开验证目标发生变化，按 `docs/testing/case-maintenance.md` 更新测试用例账本和源码 `@case` 标记。

## 6. 验证与收尾

- [ ] 6.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-adapter-sdk` 和 `docnav-markdown`。
- [ ] 6.2 运行 Markdown adapter CLI smoke 和负向矩阵测试，确认 stdout、stderr、exit code 和配置错误诊断符合契约。
- [ ] 6.3 对涉及 CLI、adapter、docs 和测试边界的最终改动运行 `bun run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 6.4 使用局部 diff 确认实现只改动 adapter direct CLI 配置相关代码、文档、测试和本 change artifacts。
- [ ] 6.5 在所有实现任务和验证任务完成后，再运行 `openspec validate implement-adapter-direct-cli-config --type change --strict --no-interactive` 并准备归档评估。
