import fs from "node:fs";
import path from "node:path";

import { tempRoot } from "../config.ts";

export interface MarkdownConfigProject {
  docRelPath: string;
  docText: string;
  docnavDir: string;
  fixturesDir: string;
  root: string;
  userConfigPath: string;
}

let projectCounter = 0;

export function createProject(name: string): MarkdownConfigProject {
  const projectRoot = path.join(tempRoot, `${String(projectCounter++).padStart(2, "0")}-${slug(name)}`);
  const docsDir = path.join(projectRoot, "docs");
  const docnavDir = path.join(projectRoot, ".docnav");
  const fixturesDir = path.join(projectRoot, "fixtures");
  for (const dir of [docsDir, docnavDir, fixturesDir]) {
    fs.mkdirSync(dir, { recursive: true });
  }
  const docText = [
    "# Top",
    "",
    "top text",
    "",
    "## Middle",
    "",
    "middle text",
    "",
    "### Deep",
    "",
    "deep-target text appears here for find.",
    ""
  ].join("\n");
  const docRelPath = "docs/config.md";
  fs.writeFileSync(path.join(projectRoot, docRelPath), docText, "utf8");
  return {
    docRelPath,
    docText,
    docnavDir,
    fixturesDir,
    root: projectRoot,
    userConfigPath: path.join(projectRoot, "docnav-markdown.json")
  };
}

export function writeProjectConfig(project: MarkdownConfigProject, value: unknown) {
  writeJson(path.join(project.docnavDir, "docnav-markdown.json"), value);
}

export function writeUserConfig(project: MarkdownConfigProject, value: unknown) {
  writeJson(project.userConfigPath, value);
}

export function writeJson(filePath: string, value: unknown) {
  fs.writeFileSync(filePath, `${JSON.stringify(value, null, 2)}\n`, "utf8");
}

function slug(value: string) {
  return value.replace(/[^a-z0-9]+/giu, "-").replace(/^-|-$/gu, "").toLowerCase();
}
