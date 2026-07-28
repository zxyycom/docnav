import fs from "node:fs";
import path from "node:path";

import ts from "typescript";

import { resolveExistingWorkspacePath } from "../relative-path.ts";

export type SmokeImportBinding = {
  importedName: string;
  moduleSpecifier: string;
};

export type SmokeSourceModule = {
  absolutePath: string;
  declarations: Map<string, ts.Node>;
  imports: Map<string, SmokeImportBinding>;
  sourceFile: ts.SourceFile;
  sourcePath: string;
};

export function readSmokeSourceModule(options: {
  workspaceRoot: string;
  sourcePath: string;
}): SmokeSourceModule {
  const resolved = resolveExistingWorkspacePath(
    options.workspaceRoot,
    options.sourcePath,
    `Smoke implementation ${options.sourcePath}`
  );
  if (!resolved.stats.isFile()) {
    throw new Error(
      `smoke implementation must be a regular file: ${options.sourcePath}`
    );
  }
  const source = fs.readFileSync(resolved.absolutePath, "utf8");
  const sourceFile = ts.createSourceFile(
    options.sourcePath,
    source,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS
  );
  return {
    absolutePath: resolved.absolutePath,
    declarations: collectTopLevelDeclarations(sourceFile),
    imports: collectImports(sourceFile),
    sourceFile,
    sourcePath: options.sourcePath
  };
}

export function collectReferencedNames(node: ts.Node): Set<string> {
  const names = new Set<string>();
  visit(node);
  return names;

  function visit(current: ts.Node): void {
    if (ts.isIdentifier(current)) {
      names.add(current.text);
    }
    ts.forEachChild(current, visit);
  }
}

export function resolveSmokeImportPath(options: {
  workspaceRoot: string;
  module: SmokeSourceModule;
  moduleSpecifier: string;
}): string | null {
  if (!options.moduleSpecifier.startsWith(".")) {
    return null;
  }
  const unresolved = path.resolve(
    path.dirname(options.module.absolutePath),
    options.moduleSpecifier
  );
  for (const candidate of [
    unresolved,
    `${unresolved}.ts`,
    path.join(unresolved, "index.ts")
  ]) {
    if (!fs.existsSync(candidate)) {
      continue;
    }
    const sourcePath = workspaceRelativePath(options.workspaceRoot, candidate);
    const resolved = resolveExistingWorkspacePath(
      options.workspaceRoot,
      sourcePath,
      `Smoke import ${options.moduleSpecifier}`
    );
    if (resolved.stats.isFile()) {
      return sourcePath;
    }
  }
  return null;
}

export function isWithinSmokeSourceRoots(
  sourcePath: string,
  roots: readonly string[]
): boolean {
  return roots.some((root) => (
    sourcePath === root ||
    sourcePath.startsWith(`${root}/`)
  ));
}

function collectTopLevelDeclarations(
  sourceFile: ts.SourceFile
): Map<string, ts.Node> {
  const declarations = new Map<string, ts.Node>();
  for (const statement of sourceFile.statements) {
    if (
      (
        ts.isFunctionDeclaration(statement) ||
        ts.isClassDeclaration(statement) ||
        ts.isEnumDeclaration(statement)
      ) &&
      statement.name
    ) {
      declarations.set(statement.name.text, statement);
      continue;
    }
    if (!ts.isVariableStatement(statement)) {
      continue;
    }
    for (const declaration of statement.declarationList.declarations) {
      if (ts.isIdentifier(declaration.name)) {
        declarations.set(declaration.name.text, declaration);
      }
    }
  }
  return declarations;
}

function collectImports(
  sourceFile: ts.SourceFile
): Map<string, SmokeImportBinding> {
  const imports = new Map<string, SmokeImportBinding>();
  for (const statement of sourceFile.statements) {
    if (
      !ts.isImportDeclaration(statement) ||
      !ts.isStringLiteral(statement.moduleSpecifier) ||
      !statement.importClause
    ) {
      continue;
    }
    addImportClause(
      imports,
      statement.importClause,
      statement.moduleSpecifier.text
    );
  }
  return imports;
}

function addImportClause(
  imports: Map<string, SmokeImportBinding>,
  clause: ts.ImportClause,
  moduleSpecifier: string
): void {
  if (clause.name) {
    imports.set(clause.name.text, {
      importedName: "default",
      moduleSpecifier
    });
  }
  const bindings = clause.namedBindings;
  if (!bindings || !ts.isNamedImports(bindings)) {
    return;
  }
  for (const element of bindings.elements) {
    imports.set(element.name.text, {
      importedName: element.propertyName?.text ?? element.name.text,
      moduleSpecifier
    });
  }
}

function workspaceRelativePath(
  workspaceRoot: string,
  absolutePath: string
): string {
  const relativePath = path.relative(path.resolve(workspaceRoot), absolutePath);
  if (
    relativePath === ".." ||
    relativePath.startsWith(`..${path.sep}`) ||
    path.isAbsolute(relativePath)
  ) {
    throw new Error(`smoke implementation is outside the checkout: ${absolutePath}`);
  }
  return relativePath.split(path.sep).join("/");
}
