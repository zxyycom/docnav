本 change 的 delta spec 定义 repository quality duplicate-code 观测迁移到 jscpd 后必须保持的可观察行为、归一化边界和非阻断语义。

当前 change 只在 `openspec/changes/replace-pmd-cpd-with-jscpd/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Duplicate-code observation uses jscpd
Repository quality observation MUST use jscpd as the configured duplicate-code scanner when duplicate detection is enabled. The scanner MUST be discovered through repository-managed package/dependency configuration, and duplicate-code observation MUST NOT require Java, PMD, or a system `pmd` command.

#### Scenario: Full profile runs jscpd duplicate detection
- **WHEN** the full quality profile runs and jscpd is available
- **THEN** the duplicate-code scan invokes jscpd for the configured scan inputs
- **THEN** the quality snapshot records jscpd tool metadata
- **THEN** duplicate-code findings are normalized before any report, warning, baseline, or verifier consumer reads them

#### Scenario: Quick profile still skips duplicate detection
- **WHEN** the quick quality profile runs
- **THEN** duplicate-code detection is skipped
- **THEN** the user-visible output continues to state that the run is not a full quality scan

#### Scenario: Missing jscpd is explicit
- **WHEN** jscpd is not available or cannot execute
- **THEN** tool availability records the missing or failed jscpd state explicitly
- **THEN** the scan does not silently report an empty successful duplicate-code result

### Requirement: jscpd output is normalized behind the repository wrapper
The repository quality wrapper MUST parse jscpd machine output into the existing duplicate-code normalized model before generating aggregates, warnings, reports, trends, cache entries, or verifier output. Third-party jscpd raw output MUST remain diagnostic material and MUST NOT become a stable downstream contract.

#### Scenario: jscpd JSON findings map to duplicate fragments
- **WHEN** jscpd reports duplicate code in machine-readable output
- **THEN** the wrapper emits duplicate fragments with token count, line count, locations, code areas, and changed-scope annotations
- **THEN** downstream quality warning and report code consumes only the normalized duplicate fragment model

#### Scenario: Raw output remains diagnostic material
- **WHEN** the quality scan writes raw scanner artifacts
- **THEN** jscpd raw output or normalization inputs are retained only as diagnostic artifacts
- **THEN** repository-owned `metrics.json`, warnings, and report output remain the stable quality observation surfaces

### Requirement: Duplicate-code scanner migration preserves quality policy boundaries
The jscpd migration MUST preserve the existing repository quality policy boundaries for code areas, per-area minimum tokens, generated/excluded files, baseline comparison, accepted warnings, and non-blocking warning behavior.

#### Scenario: Code area policies remain effective
- **WHEN** duplicate-code detection scans multiple code areas
- **THEN** each code area uses its configured duplicate-code scan threshold and exclusion boundary
- **THEN** generated and excluded files do not produce duplicate-code warning records

#### Scenario: Baseline and cache use jscpd identities
- **WHEN** duplicate-code scan results are cached or compared with a baseline
- **THEN** cache identities and baseline metadata distinguish jscpd results from previous PMD CPD results
- **THEN** stale PMD CPD cache payloads are not treated as successful jscpd scan results

#### Scenario: Duplicate-code warnings remain non-blocking
- **WHEN** jscpd duplicate-code findings generate warning records
- **THEN** standalone quality output may report warning status
- **THEN** workspace verifier warning status follows the existing accepted warning policy
- **THEN** duplicate-code metric values do not become blocking merge gates
