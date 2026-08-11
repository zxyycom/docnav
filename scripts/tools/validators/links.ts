import fs from "node:fs";
import path from "node:path";
import GithubSlugger from "github-slugger";
import MarkdownIt from "markdown-it";
import type { Token } from "markdown-it";

import { FILE_SYSTEM } from "./config.ts";
import { walk } from "./repo/files.ts";
import { root, toAbs, toRel } from "./repo/paths.ts";

const markdownParser = new MarkdownIt({
  html: true,
  linkify: false,
  typographer: false
});

export type MarkdownLinkFailure = {
  sourcePath: string;
  target: string;
  reason: "missing_path" | "missing_fragment" | "outside_root";
};

type ValidationRoots = {
  canonical: string;
  lexical: string;
};

type LocalTargetResolution =
  | { fragment: string; targetPath: string }
  | { reason: MarkdownLinkFailure["reason"] };

export function validateMarkdownLinks(): void {
  const markdownFiles = markdownFilesForLinkValidation();
  const failures = findMarkdownLinkFailures(markdownFiles, root);

  if (failures.length > 0) {
    const formatted = failures.map((failure) =>
      `${toRel(failure.sourcePath)} -> ${failure.target} (${failure.reason})`
    );
    throw new Error(`invalid markdown links:\n${formatted.join("\n")}`);
  }

  console.log(`markdown links ok: ${markdownFiles.length} file(s)`);
}

/** Validate repository-local links and GitHub-style ATX heading fragments. */
export function findMarkdownLinkFailures(
  markdownFiles: readonly string[],
  validationRoot: string
): MarkdownLinkFailure[] {
  const failures: MarkdownLinkFailure[] = [];
  const anchorCache = new Map<string, Set<string>>();
  const roots = resolveValidationRoots(validationRoot);

  for (const sourcePath of markdownFiles) {
    const { resolvedSourcePath, sourceText } = readContainedMarkdownSource(sourcePath, roots);
    for (const rawTarget of markdownLinkTargets(sourceText)) {
      const failure = validateMarkdownLinkTarget({
        anchorCache,
        rawTarget,
        resolvedSourcePath,
        roots,
        sourcePath
      });
      if (failure) failures.push(failure);
    }
  }

  return failures;
}

function resolveValidationRoots(validationRoot: string): ValidationRoots {
  const lexical = path.resolve(validationRoot);
  return { lexical, canonical: fs.realpathSync(lexical) };
}

function readContainedMarkdownSource(
  sourcePath: string,
  roots: ValidationRoots
): { resolvedSourcePath: string; sourceText: string } {
  const resolvedSourcePath = path.resolve(sourcePath);
  if (!isPathWithinRoot(resolvedSourcePath, roots.lexical)) {
    throw new Error(`markdown source is outside the validation root: ${sourcePath}`);
  }
  const canonicalSourcePath = fs.realpathSync(resolvedSourcePath);
  if (!isPathWithinRoot(canonicalSourcePath, roots.canonical)) {
    throw new Error(`markdown source resolves outside the validation root: ${sourcePath}`);
  }
  return {
    resolvedSourcePath,
    sourceText: fs.readFileSync(canonicalSourcePath, "utf8")
  };
}

function validateMarkdownLinkTarget(options: {
  anchorCache: Map<string, Set<string>>;
  rawTarget: string;
  resolvedSourcePath: string;
  roots: ValidationRoots;
  sourcePath: string;
}): MarkdownLinkFailure | null {
  if (isExternalTarget(options.rawTarget)) return null;

  const resolution = resolveLocalMarkdownTarget(
    options.rawTarget,
    options.resolvedSourcePath,
    options.roots
  );
  if ("reason" in resolution) {
    return { sourcePath: options.sourcePath, target: options.rawTarget, reason: resolution.reason };
  }
  if (resolution.fragment === "" || !isMarkdownPath(resolution.targetPath)) return null;

  const anchors = anchorsForMarkdownFile(resolution.targetPath, options.anchorCache);
  return anchors.has(decodeUriComponentSafely(resolution.fragment))
    ? null
    : { sourcePath: options.sourcePath, target: options.rawTarget, reason: "missing_fragment" };
}

function resolveLocalMarkdownTarget(
  rawTarget: string,
  resolvedSourcePath: string,
  roots: ValidationRoots
): LocalTargetResolution {
  const hashIndex = rawTarget.indexOf("#");
  const rawPath = hashIndex < 0 ? rawTarget : rawTarget.slice(0, hashIndex);
  const fragment = hashIndex < 0 ? "" : rawTarget.slice(hashIndex + 1);
  const candidatePath = rawPath === ""
    ? resolvedSourcePath
    : path.resolve(path.dirname(resolvedSourcePath), decodeUriComponentSafely(rawPath));

  if (!isPathWithinRoot(candidatePath, roots.lexical)) return { reason: "outside_root" };
  if (!fs.existsSync(candidatePath)) return { reason: "missing_path" };

  let targetPath: string;
  try {
    targetPath = fs.realpathSync(candidatePath);
  } catch {
    return { reason: "missing_path" };
  }
  return isPathWithinRoot(targetPath, roots.canonical)
    ? { fragment, targetPath }
    : { reason: "outside_root" };
}

function markdownLinkTargets(markdown: string): string[] {
  const targets: string[] = [];
  for (const token of markdownParser.parse(markdown, {})) {
    for (const child of token.children ?? []) {
      const attribute = child.type === "link_open"
        ? "href"
        : child.type === "image"
          ? "src"
          : null;
      if (attribute) {
        const target = child.attrGet(attribute);
        if (target !== null && target !== "") targets.push(String(target));
      }
    }
  }
  return targets;
}

function isExternalTarget(target: string): boolean {
  return target.startsWith("//") || /^[a-z][a-z\d+.-]*:/i.test(target);
}

function isMarkdownPath(filePath: string): boolean {
  return /\.(?:md|mdown|markdown)$/i.test(filePath);
}

function isPathWithinRoot(candidatePath: string, rootPath: string): boolean {
  const relative = path.relative(rootPath, candidatePath);
  return relative === ""
    || (!relative.startsWith(`..${path.sep}`) && relative !== ".." && !path.isAbsolute(relative));
}

function anchorsForMarkdownFile(
  filePath: string,
  cache: Map<string, Set<string>>
): Set<string> {
  const cached = cache.get(filePath);
  if (cached) return cached;

  const anchors = markdownHeadingAnchors(fs.readFileSync(filePath, "utf8"));
  cache.set(filePath, anchors);
  return anchors;
}

function markdownHeadingAnchors(markdown: string): Set<string> {
  const anchors = new Set<string>();
  const slugger = new GithubSlugger();
  const tokens = markdownParser.parse(markdown, {});

  for (let index = 0; index < tokens.length; index += 1) {
    const heading = tokens[index];
    if (heading.type !== "heading_open" || !heading.markup.startsWith("#")) continue;
    const inline = tokens[index + 1];
    if (inline?.type !== "inline") continue;

    const slug = slugger.slug(visibleInlineText(inline.children ?? []));
    if (slug !== "") anchors.add(slug);
  }

  return anchors;
}

function visibleInlineText(tokens: readonly Token[]): string {
  return tokens.map((token) => {
    switch (token.type) {
      case "text":
      case "code_inline":
      case "image":
        return token.content;
      case "softbreak":
      case "hardbreak":
        return " ";
      default:
        return "";
    }
  }).join("");
}

function decodeUriComponentSafely(value: string): string {
  try {
    return decodeURIComponent(value);
  } catch {
    return value;
  }
}

function markdownFilesForLinkValidation(): string[] {
  const markdownFiles: string[] = [];
  const lexicalRoot = path.resolve(root);
  const canonicalRoot = fs.realpathSync(lexicalRoot);
  for (const relPath of FILE_SYSTEM.markdownLinkRoots) {
    const absPath = toAbs(relPath);
    if (!fs.existsSync(absPath)) {
      throw new Error(`markdown link validation root is missing: ${relPath}`);
    }

    const canonicalPath = fs.realpathSync(absPath);
    if (
      !isPathWithinRoot(path.resolve(absPath), lexicalRoot)
      || !isPathWithinRoot(canonicalPath, canonicalRoot)
    ) {
      throw new Error(`markdown link validation root escapes the workspace: ${relPath}`);
    }

    const stat = fs.statSync(absPath);
    if (stat.isDirectory()) {
      markdownFiles.push(
        ...walk(absPath, (filePath) => filePath.endsWith(FILE_SYSTEM.markdownExtension))
      );
      continue;
    }

    if (absPath.endsWith(FILE_SYSTEM.markdownExtension)) {
      markdownFiles.push(absPath);
    }
  }

  return [...new Set(markdownFiles)].sort((left, right) =>
    toRel(left).localeCompare(toRel(right))
  );
}
