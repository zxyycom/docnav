import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  runProcess,
  writeProcessOutput
} from "../tools/foundation/src/index.ts";
import {
  expectedAstGrepVersionLine,
  runAstGrep
} from "./ast-grep.ts";

const workspaceRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  ".."
);
const fixtureRoot = fs.mkdtempSync(path.join(os.tmpdir(), "docnav-ast-grep-offline-"));

try {
  fs.writeFileSync(
    path.join(fixtureRoot, "package.json"),
    `${JSON.stringify({
      name: "docnav-ast-grep-offline-fixture",
      private: true,
      packageManager: "pnpm@11.1.3",
      devDependencies: {
        "@ast-grep/cli": "0.45.0"
      }
    }, null, 2)}\n`
  );
  fs.writeFileSync(
    path.join(fixtureRoot, "pnpm-workspace.yaml"),
    "allowBuilds:\n  '@ast-grep/cli': true\n"
  );

  for (const frozen of [false, true]) {
    if (frozen) {
      fs.rmSync(path.join(fixtureRoot, "node_modules"), {
        force: true,
        recursive: true
      });
    }
    const args = [
      "exec",
      "--",
      "pnpm",
      "--dir",
      fixtureRoot,
      "install",
      "--offline"
    ];
    if (frozen) {
      args.push("--frozen-lockfile");
    }
    const install = await runProcess({
      args,
      command: "mise",
      cwd: workspaceRoot,
      label: frozen
        ? "pnpm frozen offline install"
        : "pnpm offline lock preparation"
    });
    writeProcessOutput(install);
    if (install.status !== 0) {
      throw new Error(
        `${frozen ? "frozen " : ""}offline dependency installation failed with status ${String(install.status)}`
      );
    }
  }

  const version = await runAstGrep(["--version"], {
    cwd: fixtureRoot,
    workspaceRoot: fixtureRoot
  });
  writeProcessOutput(version);
  if (version.status !== 0 || version.stdout.trim() !== expectedAstGrepVersionLine()) {
    throw new Error(
      `offline ast-grep invocation failed: status=${String(version.status)} stdout=${JSON.stringify(version.stdout)}`
    );
  }
} finally {
  fs.rmSync(fixtureRoot, { force: true, recursive: true });
}
