# Delivery Notes

- Path: `scripts/tools/parallel-task-runner/`
- Public source entrypoint: `src/index.ts`
- Include policy: `src/**/*.ts`, `test/**/*.ts`, `README.md`, `CHANGELOG.md`, `DELIVERY.md`, `package.json`, `tsconfig.json`
- Dependency policy: source-import only from `../foundation/src/index.ts`.
- Rollback: reset the subrepo pin and Docnav caller import to the last verified runner revision.
