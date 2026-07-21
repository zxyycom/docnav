# 将仓库决策记录纳入必需验证

## 索引摘要
- 目的: 让长期决策在仓库内保持可恢复、可审计且结构一致。
- 背景: 项目已引入 decision-records skill，但个人安装和人工维护不能保证 CI 可复现或索引同步。
- 决策: 版本化管理 docs/decisions，并由 required docs validator 直接调用项目内模块执行严格检查。

## 目的
- 让会持续影响后续工作的长期判断随仓库版本化，并在本地与 CI 中及时发现目录、正文、索引和关系漂移。

## 背景
- decision-records 已作为项目级 skill 位于 `.codex/skills/decision-records`。
- 若只依赖个人安装或人工检查，不同环境可能缺少工具，`docs/decisions` 的 Markdown 与生命周期索引也可能静默失配。
- 现有 required workspace profile 已通过 `validate:docs` 承接快速、确定性的文档验证。

## 决策
- 采用: 在 `docs/decisions` 保存版本化决策集合；`validate:docs` 直接导入项目内 decision-records ESM 模块执行严格检查，并由 required workspace profile 运行该验证。
- 不采用: 不从个人 skill 目录调用工具，也不在 workspace 校验中执行需要联网的版本更新检查。
