本 tasks 是 adapter direct CLI 配置路径参数、配置读取、默认值合并、native options 显式化以及 Markdown 配置 schema/example 参考材料的正式实现清单。实现阶段按本任务清单同步 owner 主规范、代码、schema/example 和测试。

## 1. 准备审计

- [x] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“adapter direct CLI 通过 SDK 可覆盖配置路径读取自身配置，并显式化默认值/native options，同时为 Markdown 配置提供参考 schema/example”这一核心目标。
- [x] 1.2 审计 capability ID 是否正确复用 `adapter-protocol`、`docnav-contracts` 和 `markdown-navigation`，且没有创建一次性、同义或过宽 capability。
- [x] 1.3 审计当前 change 准备范围是否只包含本 change artifacts、用户确认新增的 config schema/example 参考材料和 docs validator 绑定，且没有修改现有主 specs 或实现代码。
- [x] 1.4 审计 proposal、design 和 specs 是否明确 core `docnav`、MCP 和 adapter `invoke` 不读取 adapter direct CLI 配置，且 config schema/example 不作为 runtime gate。
- [x] 1.5 准备门禁已解除；后续从 2.x 开始按本任务清单同步 owner 主规范、代码、schema/example、测试和验证材料。

## 2. 规范与验证材料同步

- [ ] 2.1 按 `docs/navigation.md` 读取 `docs/CODING_STYLE.md`、`docs/architecture.md`、`docs/cli.md`、`docs/adapter-contract.md`、`docs/adapters/markdown.md`、`docs/schemas/json-schema.md`、`docs/examples/README.md`、`docs/testing.md` 和 `docs/testing/case-maintenance.md` 中与本 change 相关的 owner 规则。
- [ ] 2.2 更新 `docs/cli.md`，记录 adapter direct CLI JSON 配置默认路径、默认用户配置目录参数、`--project-config-path` / `--user-config-path`、优先级、配置字段投影、配置源跳过 warning、help 展示和 `invoke` 不读配置。
- [ ] 2.3 更新 `docs/architecture.md`，同步 adapter direct CLI 配置所有权、项目根发现和进程边界摘要，避免只有 CLI 文档承接配置域规则。
- [ ] 2.4 更新 `docs/adapter-contract.md`，说明 adapter SDK direct CLI config path parameters、默认用户配置目录参数、config helper、标准参数来源对象、配置源 warning 和 strict invoke transport 的边界。
- [ ] 2.5 更新 `docs/adapters/markdown.md`，记录 `docnav-markdown.json` 支持 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`，并引用 Markdown config schema/example 的参考用途。
- [x] 2.6 更新 `docs/schemas/json-schema.md` 和 `docs/examples/README.md`，记录 `docnav-markdown-config.schema.json` 与 `docnav-markdown-config.json` 是配置填写、打包和文档校验参考，不是 runtime gate。
- [ ] 2.7 更新 `docs/output.md` 与 `docs/schemas/readable-common.schema.json`，把 `adapter_config_source_skipped` 作为 readable warning family 纳入稳定 warning envelope，约束其 `effect` 和 details shape。
- [ ] 2.8 更新测试策略和用例维护材料，记录 adapter direct CLI 配置优先级、配置源不可用时继续合并并输出 warning、warning schema/details、help 参数展示、invoke 不读配置和 config schema/example docs validation 的验证目标。

## 3. Adapter SDK 配置基础设施

- [ ] 3.1 在 `docnav-adapter-sdk` direct CLI 层新增配置路径参数模型，包含 adapter id、项目级配置路径默认值、默认用户配置目录参数、用户级配置路径默认值，以及 `--project-config-path` / `--user-config-path` 覆盖值；默认用户配置目录参数未提供时使用当前调用位置（启动 cwd）。
- [ ] 3.2 在 `docnav-adapter-sdk` direct CLI 层新增配置内容模型，包含 `defaults.limit_chars`、`defaults.output` 和 `options` object。
- [ ] 3.3 实现项目级配置默认路径 `.docnav/<adapter-id>.json` 与覆盖路径解析；adapter direct CLI 从启动 cwd 向上查找最近 `.docnav/` 作为项目根，未找到时使用启动 cwd，document path 不参与项目根发现；未覆盖的默认路径缺失表示没有项目级配置源，显式覆盖路径缺失或不可读表示覆盖后的项目级配置源不参与本次合并。
- [ ] 3.4 实现用户级 `<adapter-id>.json` 默认路径与覆盖路径解析：默认路径为默认用户配置目录参数下的 `<adapter-id>.json`，该目录参数未提供时使用当前调用位置（启动 cwd）；未覆盖的默认路径缺失表示没有用户级配置源，显式覆盖路径缺失或不可读表示覆盖后的用户级配置源不参与本次合并。
- [ ] 3.5 实现标准 direct CLI 参数来源对象（例如 `DirectCliParameterSources` 或等价内部类型），至少承载 `limit_chars`、`output`、`native_options`、`path`、`ref`、`query`、`page` 和 warning 列表，并作为 config helper 与既有 direct CLI 参数处理链路之间的唯一交接对象。
- [ ] 3.6 实现配置合并优先级：显式 argv > 项目级配置 > 用户级配置 > 内置默认值，并把配置投影出的 `defaults.limit_chars`、`defaults.output` 和完整 `options` object 写入标准 direct CLI 参数来源对象；`path`、`ref`、`query` 和 `page` 仍由入口参数或入口默认值提供。
- [ ] 3.7 将配置中的 `options` object 合并为标准 native options 参数来源；配置读取层不判断 key 是否注册、不校验 value 类型或范围、不判断 operation 适用性，这些继续由既有 native option 处理链路负责。
- [ ] 3.8 实现配置源读取边界，覆盖默认路径缺失、覆盖路径缺失或不可读、JSON 语法无效和顶层非 object；默认路径缺失表示没有配置源，显式覆盖路径不可用、已发现配置路径不可读、JSON 语法无效或顶层非 object 必须产生 id 为 `adapter_config_source_skipped` 且 effect 为 `operation_continued` 的 direct CLI warning；warning details 固定包含 `source_level`（`project` 或 `user`）、`path_origin`（`default` 或 `override`）、`path`（本次尝试读取的解析后路径）和 `reason_code`（`missing_override`、`not_file`、`unreadable`、`invalid_json` 或 `non_object`），并确认不可用配置源不参与本次合并，其它配置来源仍按优先级合并为标准 direct CLI 参数来源对象。
- [ ] 3.9 实现配置字段投影：配置读取层只读取 JSON object、投影 `defaults.limit_chars`、`defaults.output` 和 `options` object；未知顶层字段和未知 `defaults` 字段不产生配置读取 warning，`options` object 内的 key/value 原样进入 native options 参数来源。
- [ ] 3.10 在 `docnav-diagnostics` 和 `docnav-adapter-sdk` direct warning helper 中新增 `adapter_config_source_skipped` warning family、constructor 和 stderr formatter，并保持 readable warning envelope 的 serialized shape 稳定。
- [ ] 3.11 将标准 direct CLI 参数来源对象交给既有 direct CLI 参数处理链路完成后续标准化、类型校验、范围校验、output 枚举校验、native option 注册校验和 operation 适用性处理。

## 4. Markdown Adapter 接入

- [ ] 4.1 更新 `docnav-markdown` direct CLI wiring，传入 adapter id、默认用户配置目录参数（未提供时使用当前调用位置，即启动 cwd）、内置默认值和 native option specs 给 SDK config helper。
- [ ] 4.2 支持 `.docnav/docnav-markdown.json` 和默认用户配置目录下的 `docnav-markdown.json` 的 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`，并确认 `options.max_heading_level` 对 outline 与 find 生效。
- [ ] 4.3 确认 `--limit-chars`、`--output` 和 `--max-heading-level` 显式 argv 覆盖配置文件值。
- [ ] 4.4 确认项目级/用户级配置路径覆盖值生效，且覆盖后的默认路径不参与本次合并。
- [ ] 4.5 确认 read/info 等不适用 `max_heading_level` 的 operation 由后续 native option 处理链路决定最终行为，配置读取层只负责把 `options.max_heading_level` 放入 native options 参数来源。
- [ ] 4.6 确认 `docnav-markdown invoke`、manifest、probe 和 help 不读取 document operation 配置，并确认 document operation help 展示 `--project-config-path` / `--user-config-path`。
- [x] 4.7 确认 `docs/schemas/docnav-markdown-config.schema.json` 和 `docs/examples/json/docnav-markdown-config.json` 已存在，schema 约束 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`，示例通过 docs validator 校验，并明确它们只作为参考/打包材料。

## 5. 测试与示例覆盖

- [ ] 5.1 增加 `docnav-adapter-sdk` 单元测试，覆盖 config path defaults/override、默认用户配置目录参数及当前调用位置（启动 cwd）fallback、merge precedence、native options source object merging、未知字段投影边界和 missing file behavior。
- [ ] 5.2 增加 `docnav-markdown` CLI smoke fixture，覆盖项目级配置、用户级配置、配置路径覆盖、显式 argv 覆盖、内置默认 fallback，以及 outline/find 的 `options.max_heading_level` 行为。
- [ ] 5.3 增加矩阵测试，覆盖 JSON 语法无效、顶层非 object、显式配置路径缺失或不可读、对应 direct CLI warning details，以及读取不可用后仍继续合并其它配置来源；配置读取层只断言参数来源对象映射和未知字段投影边界，类型、范围、枚举、native option 注册和 operation 适用性由后续参数处理链路测试覆盖。
- [ ] 5.4 增加 invoke 边界测试，证明 adapter `invoke` 不读取 adapter direct CLI 配置，也不补全缺失 protocol request fields。
- [x] 5.5 增加 config schema/example docs validation，证明 `docs/examples/json/docnav-markdown-config.json` 符合 `docs/schemas/docnav-markdown-config.schema.json`，且该验证不被 runtime 读取链路依赖。
- [ ] 5.6 增加 diagnostics/readable schema 测试，证明 `adapter_config_source_skipped` 的 warning id、effect、details schema 和 stderr text line 稳定。
- [ ] 5.7 如测试函数、case 归属或公开验证目标发生变化，按 `docs/testing/case-maintenance.md` 更新测试用例账本和源码 `@case` 标记。

## 6. 验证与收尾

- [ ] 6.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-adapter-sdk` 和 `docnav-markdown`。
- [ ] 6.2 运行 Markdown adapter CLI smoke 和矩阵测试，确认配置路径覆盖、配置源不可用时继续合并并输出 warning、warning details、优先级、help 参数展示和 invoke 边界符合契约。
- [ ] 6.3 运行 `bun run validate:docs`，确认新增 config schema/example 可编译、可解析并完成示例校验。
- [ ] 6.4 对涉及 CLI、adapter、docs、schema、examples 和测试边界的最终改动运行 `bun run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 6.5 使用局部 diff 确认实现只改动 adapter direct CLI 配置相关代码、文档、schema/example、测试和本 change artifacts。
- [ ] 6.6 在所有实现任务和验证任务完成后，再运行 `openspec validate implement-adapter-direct-cli-config --type change --strict --no-interactive` 并准备归档评估。
