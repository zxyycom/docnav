import ts from "typescript";

import {
  canonicalJson,
  sha256
} from "../fingerprint.ts";
import {
  collectReferencedNames,
  isWithinSmokeSourceRoots,
  readSmokeSourceModule,
  resolveSmokeImportPath,
  type SmokeImportBinding,
  type SmokeSourceModule
} from "./smoke-source-module.ts";

type FingerprintContext = {
  allowedRoots: string[];
  modules: Map<string, SmokeSourceModule>;
  records: Map<string, {
    name: string;
    source: string;
    sourcePath: string;
    start: number;
  }>;
  visited: Set<string>;
  workspaceRoot: string;
};

export function createSmokeSourceFingerprint(options: {
  workspaceRoot: string;
  sourceRoots: readonly string[];
  sourcePath: string;
  taskSource: string;
  runExpression: string;
}): string {
  const runName = parseRunIdentifier(options.runExpression);
  const context: FingerprintContext = {
    allowedRoots: [...options.sourceRoots],
    modules: new Map(),
    records: new Map(),
    visited: new Set(),
    workspaceRoot: options.workspaceRoot
  };
  const taskModule = loadModule(context, options.sourcePath);
  const rootBinding = resolveBinding(context, taskModule, runName, true);
  collectBinding(context, rootBinding.module, rootBinding.name);

  const implementation = [...context.records.values()]
    .sort((left, right) => compareStrings(
      `${left.sourcePath}:${String(left.start).padStart(12, "0")}:${left.name}`,
      `${right.sourcePath}:${String(right.start).padStart(12, "0")}:${right.name}`
    ))
    .map(({ name, source, sourcePath }) => ({
      name,
      source: normalizeSource(source),
      sourcePath
    }));
  return sha256(canonicalJson({
    implementation,
    run: runName,
    task: normalizeSource(options.taskSource)
  }));
}

function collectBinding(
  context: FingerprintContext,
  module: SmokeSourceModule,
  name: string
): void {
  const key = `${module.sourcePath}#${name}`;
  if (context.visited.has(key)) {
    return;
  }
  const declaration = module.declarations.get(name);
  if (!declaration) {
    throw new Error(
      `smoke run binding ${name} is not declared by ${module.sourcePath}`
    );
  }
  context.visited.add(key);
  context.records.set(key, {
    name,
    source: declaration.getText(module.sourceFile),
    sourcePath: module.sourcePath,
    start: declaration.getStart(module.sourceFile)
  });

  const referencedNames = collectReferencedNames(declaration);
  for (const referencedName of [...referencedNames].sort(compareStrings)) {
    if (module.declarations.has(referencedName)) {
      collectBinding(context, module, referencedName);
      continue;
    }
    const imported = module.imports.get(referencedName);
    if (!imported) {
      continue;
    }
    collectImportedBinding(context, module, referencedName, imported);
  }
}

function collectImportedBinding(
  context: FingerprintContext,
  module: SmokeSourceModule,
  localName: string,
  imported: SmokeImportBinding
): void {
  context.records.set(`${module.sourcePath}#import:${localName}`, {
    name: localName,
    source: `${imported.moduleSpecifier}#${imported.importedName}`,
    sourcePath: module.sourcePath,
    start: -1
  });
  const targetPath = resolveSmokeImportPath({
    workspaceRoot: context.workspaceRoot,
    module,
    moduleSpecifier: imported.moduleSpecifier
  });
  if (
    !targetPath ||
    !isWithinSmokeSourceRoots(targetPath, context.allowedRoots)
  ) {
    return;
  }
  collectBinding(context, loadModule(context, targetPath), imported.importedName);
}

function resolveBinding(
  context: FingerprintContext,
  module: SmokeSourceModule,
  name: string,
  requireOwnedImplementation: boolean
): {
  module: SmokeSourceModule;
  name: string;
} {
  if (module.declarations.has(name)) {
    return { module, name };
  }
  const imported = module.imports.get(name);
  if (!imported) {
    throw new Error(
      `smoke run expression ${name} is not a top-level declaration or import in ${module.sourcePath}`
    );
  }
  const targetPath = resolveSmokeImportPath({
    workspaceRoot: context.workspaceRoot,
    module,
    moduleSpecifier: imported.moduleSpecifier
  });
  if (!targetPath) {
    throw new Error(
      `smoke run import ${imported.moduleSpecifier} cannot be resolved from ${module.sourcePath}`
    );
  }
  if (
    requireOwnedImplementation &&
    !isWithinSmokeSourceRoots(targetPath, context.allowedRoots)
  ) {
    throw new Error(
      `smoke run implementation ${targetPath} is outside smoke source roots`
    );
  }
  return {
    module: loadModule(context, targetPath),
    name: imported.importedName
  };
}

function loadModule(
  context: FingerprintContext,
  sourcePath: string
): SmokeSourceModule {
  const cached = context.modules.get(sourcePath);
  if (cached) {
    return cached;
  }
  const module = readSmokeSourceModule({
    workspaceRoot: context.workspaceRoot,
    sourcePath
  });
  context.modules.set(sourcePath, module);
  return module;
}

function parseRunIdentifier(expression: string): string {
  const sourceFile = ts.createSourceFile(
    "smoke-run-expression.ts",
    `const run = ${expression};`,
    ts.ScriptTarget.Latest,
    true,
    ts.ScriptKind.TS
  );
  const statement = sourceFile.statements[0];
  if (!statement || !ts.isVariableStatement(statement)) {
    throw new Error(`smoke run expression is invalid: ${expression}`);
  }
  const initializer = statement.declarationList.declarations[0]?.initializer;
  if (!initializer || !ts.isIdentifier(initializer)) {
    throw new Error(`smoke run expression must be an identifier: ${expression}`);
  }
  return initializer.text;
}

function normalizeSource(source: string): string {
  return source.replace(/\r\n?/gu, "\n").trim();
}

function compareStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}
