import fs from "node:fs";
import path from "node:path";

import { writeTextFile } from "../../../../scripts/tools/fs.ts";
import { parseJsonValue } from "../../../../scripts/tools/json/value.ts";
import { isRecord, isUnknownArray } from "../../../../scripts/tools/type-guards.ts";
import { fixturesDir, relativeCommand, wrapperPath } from "./project.ts";
import type { SmokeProject } from "./project.ts";
import type { RegistryAdapter } from "./registry.ts";

export interface FakeAdapter extends RegistryAdapter {
  logPath: string;
}

export interface AdapterCall {
  command?: unknown;
  [key: string]: unknown;
}

interface CreateFakeAdapterOptions {
  extensions?: string[];
  id?: string;
  mode?: string;
}

interface FakeAdapterConfig {
  extensions: string[];
  id: string;
  logPath: string;
  mode: string;
  scriptPath: string;
}

export function createRealMarkdownAdapter(project: SmokeProject, id = "docnav-markdown"): RegistryAdapter {
  const commandPath = wrapperPath(project, id);
  if (process.platform === "win32") {
    writeTextFile(
      commandPath,
      [
        "@echo off",
        "\"%DOCNAV_MARKDOWN_BIN%\" %*",
        "exit /b %ERRORLEVEL%",
        ""
      ].join("\r\n")
    );
  } else {
    writeTextFile(commandPath, ["#!/usr/bin/env sh", "exec \"$DOCNAV_MARKDOWN_BIN\" \"$@\"", ""].join("\n"));
    fs.chmodSync(commandPath, 0o755);
  }
  return {
    id,
    command: relativeCommand(project, commandPath)
  };
}

export function createFakeAdapter(project: SmokeProject, options: CreateFakeAdapterOptions = {}): FakeAdapter {
  const adapter = fakeAdapterConfig(project, options);
  const commandPath = wrapperPath(project, adapter.id);

  if (process.platform === "win32") {
    writeTextFile(
      commandPath,
      ["@echo off", fakeAdapterCommand(adapter, cmdQuote, "%*"), "exit /b %ERRORLEVEL%", ""].join("\r\n")
    );
  } else {
    writeTextFile(
      commandPath,
      ["#!/usr/bin/env sh", fakeAdapterCommand(adapter, shQuote, "\"$@\"", "exec node"), ""].join("\n")
    );
    fs.chmodSync(commandPath, 0o755);
  }

  return {
    id: adapter.id,
    command: relativeCommand(project, commandPath),
    logPath: adapter.logPath
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

function fakeAdapterConfig(project: SmokeProject, options: CreateFakeAdapterOptions): FakeAdapterConfig {
  const id = options.id ?? "fake-adapter";
  return {
    id,
    mode: options.mode ?? "valid",
    extensions: options.extensions ?? [".md", ".core"],
    logPath: path.join(project.docnavDir, `${id}-calls.jsonl`),
    scriptPath: path.join(fixturesDir, "fake-adapter.ts")
  };
}

function fakeAdapterCommand(adapter: FakeAdapterConfig, quote: (value: string) => string, forwardedArgs: string, node = "node") {
  return [
    node,
    quote(adapter.scriptPath),
    "--id",
    quote(adapter.id),
    "--mode",
    quote(adapter.mode),
    "--log",
    quote(adapter.logPath),
    "--extensions",
    quote(adapter.extensions.join(",")),
    forwardedArgs
  ].join(" ");
}

function parseAdapterCall(line: string): AdapterCall {
  const value = parseJsonValue(line, "adapter call log line");
  if (!isRecord(value) || isUnknownArray(value)) {
    throw new Error("adapter call log line must be a JSON object");
  }
  return value;
}

function cmdQuote(value: string) {
  return `"${String(value).replaceAll("\"", "\"\"")}"`;
}

function shQuote(value: string) {
  return `'${String(value).replaceAll("'", "'\\''")}'`;
}
