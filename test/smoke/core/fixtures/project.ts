import fs from "node:fs";
import path from "node:path";

import { ensureDirForFile } from "../../../../scripts/tools/fs.ts";
import { toSlashPath } from "../../../../scripts/tools/path.ts";
import { root, tempRoot } from "../config.ts";
import { writeJson } from "./registry.ts";

export interface SmokeProject {
  docnavDir: string;
  docsDir: string;
  env: NodeJS.ProcessEnv;
  normalPath: string;
  normalRelPath: string;
  root: string;
}

interface CreateProjectOptions {
  config?: unknown;
  docnavDir?: boolean;
  normalDocument?: boolean;
}

let projectCounter = 0;

export const fixturesDir = path.join(root, "test", "smoke", "core", "fixtures");
const normalDocumentFixture = path.join(fixturesDir, "normal.md");

export function createProject(name: string, options: CreateProjectOptions = {}): SmokeProject {
  const projectRoot = path.join(tempRoot, `${String(projectCounter++).padStart(2, "0")}-${slug(name)}`);
  const project = smokeProjectPaths(projectRoot);
  const userConfigDir = path.join(projectRoot, ".user-config");

  createProjectDirectories(project, userConfigDir);
  writeInitialProjectConfig(project, options);
  copyInitialNormalDocument(project, options);

  return {
    ...project,
    env: isolatedEnv(projectRoot, userConfigDir)
  };
}

export function copyNormalDocument(project: SmokeProject, relativePath: string) {
  const filePath = path.join(project.root, relativePath);
  copyFile(normalDocumentFixture, filePath);
  return toSlashPath(relativePath);
}

function smokeProjectPaths(projectRoot: string) {
  const docsDir = path.join(projectRoot, "docs");

  return {
    root: projectRoot,
    docnavDir: path.join(projectRoot, ".docnav"),
    docsDir,
    normalPath: path.join(docsDir, "normal.md"),
    normalRelPath: "docs/normal.md"
  };
}

function createProjectDirectories(project: Omit<SmokeProject, "env">, userConfigDir: string) {
  fs.mkdirSync(project.root, { recursive: true });
  fs.mkdirSync(project.docsDir, { recursive: true });
  fs.mkdirSync(userConfigDir, { recursive: true });
}

function writeInitialProjectConfig(project: Omit<SmokeProject, "env">, options: CreateProjectOptions) {
  if (options.docnavDir === false) {
    return;
  }

  fs.mkdirSync(project.docnavDir, { recursive: true });
  if (options.config !== false) {
    writeJson(path.join(project.docnavDir, "docnav.json"), options.config ?? {});
  }
}

function copyInitialNormalDocument(project: Omit<SmokeProject, "env">, options: CreateProjectOptions) {
  if (options.normalDocument !== false) {
    copyFile(normalDocumentFixture, project.normalPath);
  }
}

function copyFile(sourcePath: string, destinationPath: string) {
  ensureDirForFile(destinationPath);
  fs.copyFileSync(sourcePath, destinationPath);
}

function isolatedEnv(projectRoot: string, userConfigDir: string): NodeJS.ProcessEnv {
  return {
    DOCNAV_CONFIG_DIR: userConfigDir,
    HOME: path.join(projectRoot, ".home"),
    USERPROFILE: path.join(projectRoot, ".userprofile"),
    APPDATA: path.join(projectRoot, ".appdata"),
    XDG_CONFIG_HOME: path.join(projectRoot, ".xdg-config")
  };
}

function slug(value: string) {
  return String(value).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "") || "item";
}
