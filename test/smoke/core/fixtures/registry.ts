import fs from "node:fs";
import path from "node:path";

import { writeJsonFile, writeTextFile } from "../../../../scripts/tools/foundation/src/fs.ts";
import type { SmokeProject } from "./project.ts";

export function writeProjectConfig(project: SmokeProject, config: unknown) {
  fs.mkdirSync(project.docnavDir, { recursive: true });
  writeJson(path.join(project.docnavDir, "docnav.json"), config);
}

export function writeDamagedRegistry(project: SmokeProject) {
  fs.mkdirSync(project.docnavDir, { recursive: true });
  writeTextFile(path.join(project.docnavDir, "adapters.json"), "{ invalid json");
}

export function writeJson(filePath: string, value: unknown) {
  writeJsonFile(filePath, value);
}
