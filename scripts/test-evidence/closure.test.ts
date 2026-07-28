import assert from "node:assert/strict";
import test from "node:test";

import {
  loadTestCaseCatalog,
  validateTestCaseCoverage
} from "./cases.ts";
import { closeStaticAndRuntimeEntities } from "./closure.ts";
import {
  assertDiagnostic,
  bunEntity,
  cargoEntity,
  createCaseFixture,
  smokeEntity,
  testEntity
} from "./fixtures/catalog.ts";
import type {
  RuntimeTestEntity,
  StaticTestEntity
} from "./model.ts";

test("closes current test entities against the union of Case mappings", () => {
  assertCaseMappingClosure();
  assertStaticRuntimeClosure();
});

function assertCaseMappingClosure(): void {
  using fixture = createCaseFixture();
  const catalog = loadTestCaseCatalog({ workspaceRoot: fixture.root });
  const entities = [
    testEntity(bunEntity),
    testEntity(cargoEntity),
    testEntity(smokeEntity)
  ];

  assert.deepEqual(validateTestCaseCoverage({ catalog, entities }), []);

  const unknownEntity = "bun|tests/missing.test.ts|missing";
  const changedCatalog = {
    ...catalog,
    cases: catalog.cases.map((testCase) => (
      testCase.id === "CASE-CONTRACT-REJECT-001"
        ? {
            ...testCase,
            entityKeys: testCase.entityKeys.map((entityKey) => (
              entityKey === cargoEntity ? unknownEntity : entityKey
            ))
          }
        : testCase
    ))
  };
  const diagnostics = validateTestCaseCoverage({
    catalog: changedCatalog,
    entities
  });

  assertDiagnostic(diagnostics, "case.entity-unknown");
  assertDiagnostic(diagnostics, "entity.case-missing");
  assert.equal(
    diagnostics.some(({ code, entityKey }) => (
      code === "entity.case-missing" && entityKey === bunEntity
    )),
    false,
    "one entity may be mapped by multiple Cases"
  );
}

function assertStaticRuntimeClosure(): void {
  const identity = "tests/example.test.ts\0contract > rejects invalid input";
  const staticEntity = staticTestEntity(identity);
  const runtimeEntity = runtimeTestEntity(identity);
  const closed = closeBunEntities([staticEntity], [runtimeEntity]);
  assert.deepEqual(closed.diagnostics, []);
  assert.deepEqual(
    closed.entities.map(({ entityKey }) => entityKey),
    [bunEntity]
  );
  assertUnmatchedEntityClosure(staticEntity, runtimeEntity);
  assertDuplicateEntityClosure(staticEntity, runtimeEntity);
}

function assertUnmatchedEntityClosure(
  staticEntity: StaticTestEntity,
  runtimeEntity: RuntimeTestEntity
): void {
  const staticOnly = closeBunEntities([staticEntity], []);
  assertDiagnostic(staticOnly.diagnostics, "static-only");
  assert.match(staticOnly.diagnostics[0]?.message ?? "", /static TestEntity/);

  const runtimeOnly = closeBunEntities([], [runtimeEntity]);
  assertDiagnostic(runtimeOnly.diagnostics, "runtime-only");
  assert.match(runtimeOnly.diagnostics[0]?.message ?? "", /runtime TestEntity/);
}

function assertDuplicateEntityClosure(
  staticEntity: StaticTestEntity,
  runtimeEntity: RuntimeTestEntity
): void {
  const duplicateStatic = closeBunEntities(
    [staticEntity, { ...staticEntity }],
    [runtimeEntity]
  );
  assertDiagnostic(duplicateStatic.diagnostics, "duplicate-entity");
  assert.equal(duplicateStatic.diagnostics[0]?.origin, "static");
  assert.match(
    duplicateStatic.diagnostics[0]?.message ?? "",
    /TestEntity identity/
  );

  const duplicateRuntime = closeBunEntities(
    [staticEntity],
    [runtimeEntity, { ...runtimeEntity }]
  );
  assertDiagnostic(duplicateRuntime.diagnostics, "duplicate-entity");
  assert.equal(duplicateRuntime.diagnostics[0]?.origin, "runner");
  assert.match(
    duplicateRuntime.diagnostics[0]?.message ?? "",
    /TestEntity identity/
  );
}

function staticTestEntity(identity: string): StaticTestEntity {
  return {
    identity,
    sourcePath: "tests/example.test.ts",
    sourceRange: {
      startLine: 7,
      startColumn: 1,
      endLine: 7,
      endColumn: 42
    }
  };
}

function runtimeTestEntity(identity: string): RuntimeTestEntity {
  return {
    identity,
    target: "tests/example.test.ts",
    selector: "contract > rejects invalid input"
  };
}

function closeBunEntities(
  statics: StaticTestEntity[],
  runtime: RuntimeTestEntity[]
): ReturnType<typeof closeStaticAndRuntimeEntities> {
  return closeStaticAndRuntimeEntities({
    runner: "bun",
    statics,
    runtime,
    createEntityKey: ({ target, selector }) => `bun|${target}|${selector}`
  });
}
