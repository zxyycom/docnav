import fs from "node:fs";
import path from "node:path";

import { isRecord, isUnknownArray, parseJsonValue } from "../../../scripts/tools/types.ts";
import { root, tempRoot } from "./config.ts";

export interface SmokeProject {
  binDir: string;
  docnavDir: string;
  docsDir: string;
  env: NodeJS.ProcessEnv;
  normalPath: string;
  normalRelPath: string;
  root: string;
}

export interface RegistryAdapter {
  command: string;
  id: string;
}

export interface FakeAdapter extends RegistryAdapter {
  logPath: string;
}

export interface AdapterCall {
  command?: unknown;
  [key: string]: unknown;
}

interface CreateProjectOptions {
  config?: unknown;
  docnavDir?: boolean;
  normalDocument?: boolean;
}

interface CreateFakeAdapterOptions {
  extensions?: string[];
  id?: string;
  mode?: string;
}

let projectCounter = 0;
const fixturesDir = path.join(root, "test", "smoke", "core", "fixtures");
const normalDocumentFixture = path.join(fixturesDir, "normal.md");

export function createProject(name: string, options: CreateProjectOptions = {}): SmokeProject {
  const projectRoot = path.join(tempRoot, `${String(projectCounter++).padStart(2, "0")}-${slug(name)}`);
  const docnavDir = path.join(projectRoot, ".docnav");
  const docsDir = path.join(projectRoot, "docs");
  const binDir = path.join(projectRoot, "bin");
  const userConfigDir = path.join(projectRoot, ".user-config");
  fs.mkdirSync(projectRoot, { recursive: true });
  fs.mkdirSync(docsDir, { recursive: true });
  fs.mkdirSync(binDir, { recursive: true });
  fs.mkdirSync(userConfigDir, { recursive: true });

  if (options.docnavDir !== false) {
    fs.mkdirSync(docnavDir, { recursive: true });
    if (options.config !== false) {
      writeJson(path.join(docnavDir, "docnav.json"), options.config ?? {});
    }
  }

  if (options.normalDocument !== false) {
    copyFile(normalDocumentFixture, path.join(docsDir, "normal.md"));
  }

  const project = {
    root: projectRoot,
    docnavDir,
    docsDir,
    binDir,
    normalPath: path.join(docsDir, "normal.md"),
    normalRelPath: "docs/normal.md",
    env: isolatedEnv(projectRoot, userConfigDir)
  };

  return project;
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
  writeText(path.join(project.docnavDir, "adapters.json"), "{ invalid json");
}

export function copyNormalDocument(project: SmokeProject, relativePath: string) {
  const filePath = path.join(project.root, relativePath);
  copyFile(normalDocumentFixture, filePath);
  return relativePath.replaceAll(path.sep, "/");
}

export function createRealMarkdownAdapter(project: SmokeProject, id = "docnav-markdown"): RegistryAdapter {
  const commandPath = wrapperPath(project, id);
  if (process.platform === "win32") {
    writeText(
      commandPath,
      [
        "@echo off",
        "\"%DOCNAV_MARKDOWN_BIN%\" %*",
        "exit /b %ERRORLEVEL%",
        ""
      ].join("\r\n")
    );
  } else {
    writeText(commandPath, ["#!/usr/bin/env sh", "exec \"$DOCNAV_MARKDOWN_BIN\" \"$@\"", ""].join("\n"));
    fs.chmodSync(commandPath, 0o755);
  }
  return {
    id,
    command: relativeCommand(project, commandPath)
  };
}

export function createFakeAdapter(project: SmokeProject, options: CreateFakeAdapterOptions = {}): FakeAdapter {
  const id = options.id ?? "fake-adapter";
  const mode = options.mode ?? "valid";
  const extensions = options.extensions ?? [".md", ".core"];
  const logPath = path.join(project.docnavDir, `${id}-calls.jsonl`);
  const commandPath = wrapperPath(project, id);
  const fakeAdapterScript = path.join(fixturesDir, "fake-adapter.ts");

  if (process.platform === "win32") {
    writeText(
      commandPath,
      [
        "@echo off",
        [
          "node",
          cmdQuote(fakeAdapterScript),
          "--id",
          cmdQuote(id),
          "--mode",
          cmdQuote(mode),
          "--log",
          cmdQuote(logPath),
          "--extensions",
          cmdQuote(extensions.join(",")),
          "%*"
        ].join(" "),
        "exit /b %ERRORLEVEL%",
        ""
      ].join("\r\n")
    );
  } else {
    writeText(
      commandPath,
      [
        "#!/usr/bin/env sh",
        [
          "exec node",
          shQuote(fakeAdapterScript),
          "--id",
          shQuote(id),
          "--mode",
          shQuote(mode),
          "--log",
          shQuote(logPath),
          "--extensions",
          shQuote(extensions.join(",")),
          "\"$@\""
        ].join(" "),
        ""
      ].join("\n")
    );
    fs.chmodSync(commandPath, 0o755);
  }

  return {
    id,
    command: relativeCommand(project, commandPath),
    logPath
  };
}

export function readAdapterCalls(adapter: FakeAdapter): AdapterCall[] {
  if (!fs.existsSync(adapter.logPath)) {
    return [];
  }
  return fs
    .readFileSync(adapter.logPath, "utf8")
    .split(/\r?\n/)
    .filter((line) => line.trim().length > 0)
    .map(parseAdapterCall);
}

function parseAdapterCall(line: string): AdapterCall {
  const value = parseJsonValue(line, "adapter call log line");
  if (!isRecord(value) || isUnknownArray(value)) {
    throw new Error("adapter call log line must be a JSON object");
  }
  return value;
}

export function writeJson(filePath: string, value: unknown) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function writeText(filePath: string, content: string) {
  fs.mkdirSync(path.dirname(filePath), { recursive: true });
  fs.writeFileSync(filePath, content, "utf8");
}

function copyFile(sourcePath: string, destinationPath: string) {
  fs.mkdirSync(path.dirname(destinationPath), { recursive: true });
  fs.copyFileSync(sourcePath, destinationPath);
}

function wrapperPath(project: SmokeProject, id: string) {
  const extension = process.platform === "win32" ? ".cmd" : "";
  return path.join(project.binDir, `${slug(id)}${extension}`);
}

function relativeCommand(project: SmokeProject, commandPath: string) {
  return path.relative(project.root, commandPath).replaceAll(path.sep, "/");
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

function cmdQuote(value: string) {
  return `"${String(value).replaceAll("\"", "\"\"")}"`;
}

function shQuote(value: string) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}
