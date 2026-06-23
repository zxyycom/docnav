本 change 目标是统一 core `docnav` 和 `docnav-adapter-sdk` direct CLI 的标准参数定义机制，让两边都使用 builder 风格的 Rust 参数定义对象驱动 CLI flag、help、配置路径、校验、来源合并和 schema metadata；本文档只是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 tasks，不影响现有其它文档或主规范。

## 1. 审计门禁

- [ ] 1.1 阻塞级审计：在执行任何实现任务前，审计 proposal、design、specs 和 tasks 是否都围绕“统一 core 与 SDK 的共享标准参数定义模型”这一核心句；确认 capability ID 只复用 `core-cli` 和 `adapter-protocol`；确认本 change 只包含 `openspec/changes/unify-standard-parameter-definitions/` 下未审核临时 artifacts，尚未修改主规范、schema、examples 或代码；确认没有引入具体业务参数变更。审计未完成前不得执行 2.x 及后续任何实现任务。

执行边界：除 1.1 外，所有实现、文档、测试和验证任务都以 1.1 完成为前置条件。

## 2. 主规范和验证材料

- [ ] 2.1 更新 `docs/architecture.md` 中配置所有权和共享 helper 说明，明确共享标准参数 definition model 可被 core 与 SDK 复用，但参数注册集合仍由各 owner 分别拥有。
- [ ] 2.2 更新 `docs/cli.md`，说明 core CLI 标准参数由共享 definition model 驱动 `config get/set/unset/list`、document argv、help/default 文案、document context 输出和 typed validation。
- [ ] 2.3 更新 `docs/adapter-contract.md`，说明 SDK direct CLI 标准参数由同一 definition model 驱动 config projection、argv parsing、help/default 文案、typed validation、operation 参数生成和 schema metadata。
- [ ] 2.4 更新 `docs/testing.md`、`docs/testing/cases.md` 或相邻验证说明，记录 core/SDK 同名 canonical key 语义一致、definition-driven surface 不漂移、invoke 不接收 definition metadata。

## 3. 共享定义模型

- [ ] 3.1 在 `docnav-cli-args` 或等价共享 Rust 层实现标准参数 definition model，支持 builder-style 链式声明 canonical key、config file path、可选 flag、help/default 文案、value kind、parser/validator、operation applicability、source priority、default provider、finalization rule 和 schema metadata。
- [ ] 3.2 实现定义集合查询能力，使调用方可以按 canonical key、flag、config path 和 operation 过滤定义。
- [ ] 3.3 实现参数来源模型或适配层，使 explicit argv、项目配置、用户配置和内置默认值能按定义声明的优先级合并，并保留最终值来源。
- [ ] 3.4 实现 schema metadata 输出能力，至少能表达 JSON path、type、enum、minimum/maximum、description 和 default 的生成输入。
- [ ] 3.5 为共享定义模型补充 Rust 单元测试，覆盖 builder 声明、重复 key/flag/path 检测、source priority、operation applicability 和 schema metadata。

## 4. Core CLI 接入

- [ ] 4.1 将 core-owned 标准参数迁移到共享定义模型，至少覆盖当前 `defaults.adapter`、`defaults.output` 或其当前等价参数。
- [ ] 4.2 更新 core document argv parsing 和 help/default 文案，使其消费 core 标准参数定义集合。
- [ ] 4.3 更新 core config supported keys、`config get/set/unset/list` 和配置验证，使其消费 core 标准参数定义集合。
- [ ] 4.4 更新 `config list --path ... --operation ...` 或等价 document context 输出，使其展示由定义集合解析出的最终值和来源。
- [ ] 4.5 补充 core CLI 单元测试或 smoke，证明同一 core 参数的 flag/config/help/context/validation 都来自定义集合，迁移不改变当前 observable behavior。

## 5. Adapter SDK 接入

- [ ] 5.1 将 SDK direct CLI 标准参数迁移到共享定义模型，至少覆盖当前 `defaults.output` 和 SDK-owned 配置路径参数的 metadata。
- [ ] 5.2 更新 SDK direct CLI config projection，使已注册标准参数的 config path 由定义集合驱动，`options` object 仍走 native option pass-through。
- [ ] 5.3 更新 SDK direct CLI argv parsing 和 help/default 文案，使其消费 SDK 标准参数定义集合。
- [ ] 5.4 更新 SDK direct CLI typed validation、warning 和 operation 参数生成，使其从定义驱动的参数来源对象生成最终 operation 参数。
- [ ] 5.5 补充 SDK tests，证明标准参数 definition metadata 不进入 adapter `invoke` request，invoke 不读取 direct CLI 配置，native options 不被提升为标准参数。

## 6. 验证

- [ ] 6.1 运行格式化和与改动范围匹配的 Rust 单元测试，至少覆盖 `docnav-cli-args`、`docnav` core config/args、`docnav-adapter-sdk` direct args/config。
- [ ] 6.2 运行 core CLI 与 adapter direct CLI 的 focused smoke，证明迁移后当前配置 key、flag、help 和输出行为保持不变。
- [ ] 6.3 运行 schema/example 或等价 docs 验证，确认本 change 没有意外修改 protocol request/result shape。
- [ ] 6.4 若改动跨共享 crate、core、SDK 和 docs，最终优先运行 `bun run verify:docnav-workspace`；无法运行时记录原因和未覆盖风险。
- [ ] 6.5 用局部 diff 确认修改范围保持在标准参数定义机制相关的规范、共享 helper、core、SDK 和测试文件。
