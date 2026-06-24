本 tasks 定义配置命中的非结构化文档在 `outline` 时直接全文读取的实现入口；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 1. 阻塞级审计

- [ ] 1.1 审计 proposal、design、specs 和 tasks 是否都围绕“配置命中的非结构化文档在 `outline` 时直接返回全文内容，不再要求 ref 或分页”这一核心目标。
- [ ] 1.2 审计 capability ID 是否正确复用 `core-cli`、`adapter-protocol`、`docnav-contracts`、`markdown-navigation` 和 `readable-view-output`，且没有创建一次性、同义或过宽 capability。
- [ ] 1.3 审计当前 change 是否只包含 `openspec/changes/outline-unstructured-full-read/` 下的未审核临时 artifacts，且没有修改现有 specs、docs、schemas、examples 或实现代码。
- [ ] 1.4 审计 proposal、design 和 specs 是否明确非结构化配置命中时不返回 `doc:full`、其它 ref、page 或 continuation。
- [ ] 1.5 在 1.1-1.4 全部完成前，不得执行任何实现任务、主规范更新、示例更新、测试更新或代码改动。

## 2. 规范与验证材料同步

- [ ] 2.2 更新 `docs/protocol.md`，把 `OutlineResult` 定义为 structured entries 和 configured unstructured full content 两个可判别形态。
- [ ] 2.3 更新 `docs/cli.md` 的 document command 行为，记录非结构化 outline 生效配置语义、输出和非分页边界；具体配置文件、格式和合并方式由配置能力文档定义。
- [ ] 2.4 更新 `docs/output.md`，记录普通 outline 无 block、非结构化 outline 使用 `/content` block 的 readable-view 行为。
- [ ] 2.5 更新 `docs/adapters/markdown.md`，记录 `docnav-markdown` 非结构化 outline 配置、direct CLI 行为和 `doc:full` fallback 的边界差异。

## 3. 协议、Schema 与输出类型

- [ ] 3.1 更新 shared protocol/result 类型，使 outline success result 支持 structured 和 unstructured 两个分支。
- [ ] 3.3 更新 readable payload 类型和 output mapping，使 non-structured outline result 可以进入 readable-json、readable-view 和 protocol-json。
- [ ] 3.4 更新 readable-view renderer config 或 view kind 映射，为非结构化 outline 声明 `/content` block，同时保持普通 outline 无 block。

## 4. Core CLI 与 Markdown Adapter 实现

- [ ] 4.1 在 core outline 入口接入生效非结构化策略判断；不在本 change 固化具体配置文件、格式或合并方式。
- [ ] 4.2 在 `docnav outline` execution pipeline 中加入配置命中分支，直接读取 UTF-8 原文全文并产出 unstructured outline result。
- [ ] 4.3 确认非结构化分支不生成 adapter ref、不返回 page、不使用 `limit_chars` 裁剪内容，并输出稳定原因字段。
- [ ] 4.4 在 `docnav-markdown` 中接入非结构化 outline 的生效配置/operation option，并让 direct CLI 三种输出模式与 core CLI shape 一致。
- [ ] 4.5 保持未命中配置的 Markdown outline、`doc:full` fallback、read/find/info 行为不变。


- [ ] 5.1 增加 core CLI 行为测试，覆盖配置命中、未命中和 readable/protocol 输出中的稳定原因说明。
- [ ] 5.2 增加 protocol/readable schema 示例，覆盖普通 structured outline 和 configured unstructured outline 两种 shape。
- [ ] 5.3 增加 Markdown direct CLI smoke fixture，验证非结构化 Markdown outline 在 readable-json、readable-view 和 protocol-json 下不含 entries、ref 或 page。
- [ ] 5.5 增加 readable-view conformance 或等价测试，验证非结构化 outline `/content` block payload 与 readable-json content 一致。

## 6. 验证与收尾

- [ ] 6.1 运行 Rust 格式化和相关 crate 测试，至少覆盖 `docnav-protocol`、`docnav-readable`、`docnav-output`、`docnav` core 和 `docnav-markdown`。
- [ ] 6.3 对涉及 CLI、adapter、schema/example 和 docs 边界的最终改动运行 `bun run verify:docnav-workspace`，除非有明确、记录在最终说明中的环境阻塞。
- [ ] 6.4 使用局部 diff 确认实现只改动非结构化 outline 相关代码、文档、示例、测试和本 change artifacts。
- [ ] 6.5 在所有实现任务和验证任务完成后，再运行 `openspec validate outline-unstructured-full-read --type change --strict --no-interactive` 并准备归档评估。
