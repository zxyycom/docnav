# 发布包验证

本文记录 release package 的本地预验收和 CI/CD 验证边界。常规开发验证入口见 [测试策略](../testing.md)。

## 制品形状

正式发布制品由 `pnpm run package:docnav -- --target <triple>` 生成，落在 `artifacts/docnav/v<version>/<target>/package/`。

该目录只包含：

- `docnav`
- `docnav-markdown`
- `manifest.json`
- `SHA256SUMS.txt`

仓库脚本不生成 `.zip`、`.tar.gz` 或其它归档包。

`manifest.json` 是 release artifact manifest，不复用 adapter manifest schema。发布制品验证先从该清单定位文件集合，再检查大小和校验和，最后直接运行 `package/` 中的二进制，而不是回退到 `target/`、日志、临时目录或解压产物。

## 本地预验收

本地预验收通常按下面顺序跑：

```bash
pnpm run package:docnav
pnpm run verify:docnav-package
pnpm run smoke:docnav-package
```

发布包验证和 smoke 命令会自动定位当前 workspace 版本与 host target 对应的 package。使用 `--target <triple>` 选择当前版本的其它 target；使用 `--manifest <path>` 验证显式 package。`pnpm run info:docnav-package` 可打印自动定位结果。

`package:docnav` 在生成结束时校验文件集合、manifest、大小和校验和，但不运行 CLI smoke。`smoke:docnav-package` 直接测试 package 中的可执行文件。

## CI/CD 边界

CI/CD 正式制品流程必须在干净 checkout 上生成并保存 package 目录，验证时必须看到 `source_dirty: false` 和 `producer.kind: "github-actions"`，然后按 `version` 与 `target` 上传对应的 `package/` 文件集合。
