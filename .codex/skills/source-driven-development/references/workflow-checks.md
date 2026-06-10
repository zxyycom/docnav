# 工作流检查 (Workflow Checks)

## Version Detection

常见入口：

| Stack | Files |
| --- | --- |
| Node、React、Vue、Angular、Svelte | `package.json`、lockfile |
| Python、Django、Flask、FastAPI | `pyproject.toml`、`requirements.txt`、lockfile |
| Rust | `Cargo.toml`、`Cargo.lock` |
| Go | `go.mod`、`go.sum` |
| PHP、Symfony、Laravel | `composer.json`、`composer.lock` |
| Ruby、Rails | `Gemfile`、`Gemfile.lock` |

读取版本时只聚焦当前任务相关 dependency。若版本范围过宽，例如 `^`、`~` 或 workspace override，必要时检查 lockfile。

## Implementation Checks

实现前检查：

1. API signature 是否来自当前版本 docs。
2. 示例 pattern 是否适用于项目的 runtime、build tool 和 framework mode。
3. 是否存在 deprecation、migration 或 breaking change note。
4. 推荐 pattern 是否需要额外 provider、plugin、config 或 polyfill。
5. 浏览器或 runtime support 是否满足项目目标。

实现中检查：

1. 使用项目已有 helper、adapter 和 style。
2. 把 source-driven pattern 映射到本仓库边界，不复制 unrelated example architecture。
3. 不因 docs 示例简化而省略项目需要的 error handling、typing 或 tests。
4. 对无法直接验证的 glue code 标记假设并用测试覆盖。

## Red Flags

出现任一信号时回到 source：

1. 准备凭记忆写 framework-specific code。
2. 只知道 API 名称，不知道当前版本 signature。
3. 使用 “I believe” 或 “应该” 解释关键 pattern。
4. 引用 blog、tutorial 或 Q&A 作为最终依据。
5. 没检查 dependency file 就决定 implementation pattern。
6. 发现 deprecated API 仍被复制进新代码。
7. 最终回复没有列出 sources。

## Final Verification

交付前确认：

1. Source URLs 能打开并指向具体 section。
2. 所有关键 pattern 都能追溯到 official source。
3. 代码、测试和文档没有引用互相冲突的版本假设。
4. 用户能区分 verified、project-convention 和 unverified 部分。
5. 若任务涉及 Docnav contract，已按 `docs/navigation.md` 读取并同步对应主规范、schema、examples 或 tests。
