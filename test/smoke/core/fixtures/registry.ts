import fs from "node:fs";
import path from "node:path";

import { writeJsonFile, writeTextFile } from "../../../../scripts/tools/fs.ts";
import type { SmokeProject } from "./project.ts";

export interface RegistryAdapter {
  command: string;
  id: string;
}

export function writeProjectConfig(project: SmokeProject, config: unknown) {
  fs.mkdirSync(project.docnavDir, { recursive: true });
  writeJson(path.join(project.docnavDir, "docnav.json"), config);
}

export function writeRegistry(project: SmokeProject, adapters: readonly RegistryAdapter[]) {
  fs.mkdirSync(project.docnavDir, { recursive: true });
  writeJson(path.join(project.docnavDir, "adapters.json"), {
    version: 1,
    adapters: adapters.map((adapter) => ({
      id: adapter.id,
      command: adapter.command
    }))
  });
}

export function writeDamagedRegistry(project: SmokeProject) {
  fs.mkdirSync(project.docnavDir, { recursive: true });
  writeTextFile(path.join(project.docnavDir, "adapters.json"), "{ invalid json");
}

export function writeJson(filePath: string, value: unknown) {
  writeJsonFile(filePath, value);
}
