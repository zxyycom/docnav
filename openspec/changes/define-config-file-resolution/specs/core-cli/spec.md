## 一句话核心

`docnav` core CLI SHALL define explicit user/project config file flags and default path resolution; 当前 change 只在 `openspec/changes/define-config-file-resolution/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 文档状态

本 spec delta 只描述 `core-cli` capability 的新增要求。它不修改配置 JSON 字段形状、adapter option 语义、protocol/readable 输出 shape 或 selected adapter dispatch 规则。

## ADDED Requirements

### Requirement: Core CLI resolves config file paths from explicit flags and defaults

`docnav` core CLI MUST accept `--user-config <path>` and `--project-config <path>` as strict public input for commands that read, write or inspect Docnav configuration. These flags MUST identify exact config JSON file paths for the current invocation. User config file path resolution MUST use `--user-config` first, then `DOCNAV_CONFIG_DIR/docnav.json`, then the documented platform user default under `.docnav/docnav.json`. Project config file path resolution MUST use `--project-config` first, then the current project context's `.docnav/docnav.json`. These path-resolution rules MUST NOT change navigation parameter source priority `explicit > project > user > built_in`.

#### Scenario: Document operation supplies explicit config files

- **WHEN** a caller executes `docnav outline docs/guide.md --project-config ./fixtures/project.json --user-config ./fixtures/user.json`
- **THEN** core resolves both flag values as config file paths for this invocation
- **THEN** core supplies those paths as the project and user config source descriptors to `docnav-navigation`
- **THEN** core does not derive project config from the project context default for this invocation
- **THEN** core does not derive user config from `DOCNAV_CONFIG_DIR` or platform defaults for this invocation

#### Scenario: User config falls back through environment then platform default

- **WHEN** a caller omits `--user-config`
- **AND** `DOCNAV_CONFIG_DIR` is set
- **THEN** core resolves the user config file path as `DOCNAV_CONFIG_DIR/docnav.json`
- **AND** when `DOCNAV_CONFIG_DIR` is not set
- **THEN** core resolves the user config file path to the documented platform user default under `.docnav/docnav.json`

#### Scenario: Project config falls back to project context default

- **WHEN** a caller omits `--project-config`
- **THEN** core resolves the project config file path as the current project context's `.docnav/docnav.json`
- **THEN** existing project context rules continue to decide the project context

#### Scenario: Config commands read and write selected config files

- **WHEN** a caller executes `docnav config set defaults.output readable-json --project-config ./fixtures/project.json`
- **THEN** core writes the project-scope value to `./fixtures/project.json`
- **AND** when a caller executes `docnav config set defaults.pagination.limit 321 --user --user-config ./fixtures/user.json`
- **THEN** core writes the user-scope value to `./fixtures/user.json`
- **AND** when a caller executes `docnav config list --path docs/guide.md --operation outline --project-config ./fixtures/project.json --user-config ./fixtures/user.json`
- **THEN** the document context is resolved using those selected project and user config files

#### Scenario: Init and doctor use selected config paths

- **WHEN** a caller executes `docnav init --project-config ./fixtures/project.json`
- **THEN** core creates or preserves `./fixtures/project.json` as the project config file for that invocation
- **AND** when a caller executes `docnav doctor --project-config ./fixtures/project.json --user-config ./fixtures/user.json`
- **THEN** doctor checks those selected config files instead of unrelated defaults

#### Scenario: Config path flags remain strict CLI input

- **WHEN** a caller provides an unknown config path flag, a config path flag without a value, or a config path flag on a command where it is not documented
- **THEN** core returns an input diagnostic before reading config sources or dispatching document operations
- **THEN** the invalid argv is not ignored or treated as a config JSON field
