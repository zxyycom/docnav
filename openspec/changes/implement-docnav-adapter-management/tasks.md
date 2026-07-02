本 tasks 已按 `adopt-core-linked-adapter-libraries` 收敛；动态 adapter 制品管理不再进入默认实现范围。

## 1. Static Inspection Surface

- [ ] 1.1 更新 CLI/docs/spec，将 `docnav adapter list` 定义为 core release static registry inspection。
- [ ] 1.2 删除或拒绝 `adapter install/register/update/remove` 默认 CLI commands。
- [ ] 1.3 更新 doctor，使其检查 static registry 和 adapter layer 可用性。

## 2. Historical Material Cleanup

- [ ] 2.1 删除用户级安装 registry、项目级 adapter 策略 registry、managed artifact、fingerprint 和 command path 相关默认实现目标。
- [ ] 2.2 更新 schema/examples/fixtures，不再将 installed adapter 或 install guidance 作为默认修复路径。
- [ ] 2.3 覆盖 historical `.docnav/adapters.json` 不参与默认 document operation implementation source。

## 3. Validation

- [ ] 3.1 运行 docs/schema/case catalog 验证。
- [ ] 3.2 运行 core CLI parser/smoke 验证，覆盖 `adapter list` 和 dynamic command removal。
