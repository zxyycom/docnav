**一句话核心：adapter management 的长期方向被替换为 static adapter inspection，避免默认路径读取或维护外部 adapter 制品。**

## Context

原 proposal 计划维护用户级安装 registry、项目级 id/version 策略 registry、managed artifact 目录、command path 和 fingerprint。`adopt-core-linked-adapter-libraries` 已改变边界：默认 adapter set 随 core release 编译，`docnav` 只通过 static registry 选择 adapter library handle。

## Goals / Non-Goals

**Goals:**

- 记录动态 adapter management 目标被取消。
- 保留 `docnav adapter list` 作为 static registry inspection。
- 明确 `install/register/update/remove` 不进入默认 CLI surface。

**Non-Goals:**

- 不实现 adapter 下载、托管安装、本地路径登记、fingerprint 或 artifact cleanup。
- 不让项目配置或用户配置提供 adapter implementation source。
- 不改变 adapter-owned parser、ref、navigation 和 native option 语义。

## Decisions

1. Static registry 是默认 implementation source 边界。
   - `docnav` 只遍历 core release 内置 adapter records。
   - Project/user config 和 CLI 只能选择 registry 中已有 adapter id，不能提供 executable 或 command path。

2. `adapter list` 是 inspection，不是 management。
   - 输出 adapter id、version 和 format metadata。
   - 不读取 `.docnav/adapters.json`、用户级安装 registry、managed artifact record 或 fingerprint。

3. Dynamic management commands 被删除。
   - `install/register/update/remove` 作为默认命令必须失败。
   - 后续若重新引入动态 adapter distribution model，必须新开 change 并重新审计 security、distribution 和 compatibility 边界。

## Risks / Trade-offs

- [Risk] 失去运行时扩展 adapter 的灵活性。→ Mitigation: 当前 v0 优先稳定 core release contract；新增 adapter 通过 workspace crate 进入 release。
- [Risk] 旧文档或 fixture 继续暗示安装修复路径。→ Mitigation: docs/schema/examples/smoke 同步改为 static registry guidance。

## Migration Plan

1. 删除默认路径对 historical adapter registration material 的读取。
2. 将 `adapter list` 实现为 static registry metadata inspection。
3. 更新 docs、tests、schema/examples 和相关 OpenSpec change。

## Open Questions

无。
