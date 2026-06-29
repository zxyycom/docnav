本 tasks 定义用 `bpaf` 替换 clap argv frontend 的执行顺序和验收入口。

## 1. 审计门禁

- [ ] 1.1 审计当前 clap usage：列出 direct CLI parsing、help rendering、defaults、possible values、value parser 和 validation 的现有 owner。
- [ ] 1.2 确认 `bpaf` 只作为实现依赖进入 direct CLI frontend；CLI 可观察 contract 必须写成行为而不是 crate 名。
- [ ] 1.3 确认 frontend 输出模型只包含 command/subcommand、positionals、raw flag values、help request 和 frontend diagnostics。
- [ ] 1.4 确认标准参数流程或 owning native option handler 继续拥有参数语义、默认值、operation applicability 和 strict validation。
- [ ] 1.5 确认 help 输出、loose warning 行为和 adapter native options 都有测试或 smoke 验证入口。

## 2. 轮廓实现

- [ ] 2.1 引入 `bpaf` 依赖，并限制在 direct CLI frontend 层使用。
- [ ] 2.2 将 `docnav-cli-args` 保留为 frontend boundary，或明确创建新的 thin frontend crate。
- [ ] 2.3 用 `bpaf` 构建 command/subcommand、positionals、raw flag values 和 help request 的 frontend mapping。
- [ ] 2.4 将 core CLI 的 document/config/utility 入口切到新的 frontend mapping。
- [ ] 2.5 将 adapter direct CLI 的 protocol/probe/document operation 入口切到新的 frontend mapping。
- [ ] 2.6 将 `bpaf` help surface 连接到 standard parameter metadata、command context 和 owner native option metadata。
- [ ] 2.7 保留 unknown argv、extra positional、unused operation flag 的兼容 warning 投影。

## 3. 验证

- [ ] 3.1 添加或更新 unused known flag 不 eager validate 的对照测试。
- [ ] 3.2 添加或更新 actual consumed parameter 仍 strict validate 的对照测试。
- [ ] 3.3 添加或更新 core CLI help 覆盖 usage/defaults/possible values 的对照测试。
- [ ] 3.4 添加或更新 adapter direct CLI help 覆盖 adapter native options 的对照测试。
- [ ] 3.5 运行与 core CLI 和 adapter direct CLI 匹配的 smoke 验证。
