# Proposal 09: CI/CD Pipeline Integration

## Problem

TypeMill analysis tools need better integration with CI/CD pipelines:

1. **Exit Codes**: No non-zero exit codes on failures for CI to detect
2. **Output Format**: Human-readable output not CI-friendly
3. **Thresholds**: No way to fail CI based on quality thresholds
4. **Performance**: No optimizations for headless environments
5. **Reporting**: No standard CI report formats (JUnit XML, JSON)

Example CI workflows that don't work:
- Run quality analysis and fail PR if complexity > 15
- Generate analysis report for GitHub Actions
- Cache analysis results across CI runs
- Set different thresholds for CI vs local development

## Solution(s)

### 1. Exit Code Strategy

Return meaningful exit codes:

```rust
pub enum ExitCode {
    Success = 0,           // All checks passed
    Warnings = 1,          // Findings below threshold
    Failures = 2,          // Findings exceed threshold
    Error = 3,             // Tool error (LSP crash, etc.)
}
```

### 2. CI-Optimized Output Formats

Add JSON output for parsing:

```bash
# JSON output for CI tools
mill analyze quality --format json src/

# JUnit XML for test reporting
mill analyze quality --format junit src/

# GitHub Actions annotations
mill analyze quality --format github-actions src/
```

### 3. Threshold-Based Validation

Fail CI based on configurable thresholds:

```toml
# .typemill/ci.toml
[validation]
fail_on_error = true
fail_on_warning = false
max_complexity = 15
max_findings = 10
min_confidence = 0.7
```

```bash
# Use thresholds in CI
mill analyze quality --ci --fail-threshold .typemill/ci.toml src/
```

### 4. CI Mode Optimizations

Optimize for headless environments:

```rust
pub struct CIConfig {
    pub disable_colors: bool,
    pub disable_progress: bool,
    pub enable_caching: bool,
    pub parallel: bool,
}
```

### 5. Report Generation

Generate CI-friendly reports:

```bash
# Generate report artifact
mill analyze quality --report-file analysis-report.json src/

# Generate badge data
mill analyze quality --badge-output badges.json src/
```

### 6. Incremental Analysis

Analyze only changed files in CI:

```bash
# Analyze changes in PR
mill analyze quality --changed-files $(git diff --name-only main)

# Analyze with git integration
mill analyze quality --git-diff main src/
```

## Checklists

### Exit Codes
- [ ] Define exit code enum with meaningful values
- [ ] Return exit code based on analysis results
- [ ] Add `--fail-on-warnings` flag
- [ ] Add `--fail-on-errors` flag
- [ ] Add `--exit-zero` flag to always return 0

### Output Formats
- [ ] Implement JSON output format
- [ ] Implement JUnit XML output format
- [ ] Implement GitHub Actions annotations format
- [ ] Implement GitLab Code Quality report format
- [ ] Add `--format` parameter to all analysis tools

### Threshold Configuration
- [ ] Define CI threshold config schema
- [ ] Implement threshold validation logic
- [ ] Add `--fail-threshold` parameter
- [ ] Add `--ci` flag for preset CI configuration
- [ ] Support environment variable thresholds

### CI Optimizations
- [ ] Add `--no-color` flag for plain output
- [ ] Add `--no-progress` flag to disable progress bars
- [ ] Enable parallel analysis by default in CI mode
- [ ] Optimize LSP server startup for CI
- [ ] Add `--ci` flag to enable all optimizations

### Report Generation
- [ ] Add `--report-file` parameter for JSON reports
- [ ] Add report schema documentation
- [ ] Generate summary statistics in reports
- [ ] Add timestamp and metadata to reports
- [ ] Support multiple output formats simultaneously

### Git Integration
- [ ] Add `--changed-files` parameter
- [ ] Add `--git-diff <ref>` parameter
- [ ] Implement file change detection
- [ ] Cache results keyed by git commit SHA
- [ ] Support PR-specific analysis

### Documentation
- [ ] Document CI integration guide
- [ ] Add GitHub Actions workflow examples
- [ ] Add GitLab CI pipeline examples
- [ ] Add Jenkins pipeline examples
- [ ] Document exit codes and their meanings

### Testing
- [ ] Test exit codes for various scenarios
- [ ] Test JSON output parsing
- [ ] Test JUnit XML format validity
- [ ] Test GitHub Actions annotation format
- [ ] Test threshold validation
- [ ] Test incremental analysis with git

## Success Criteria

1. **Exit Codes**: CI detects failures via non-zero exit codes
2. **JSON Output**: Machine-readable JSON format parses correctly
3. **Thresholds Work**: CI fails when quality thresholds exceeded
4. **Format Support**: JUnit XML and GitHub Actions formats generate correctly
5. **Performance**: CI mode runs faster than interactive mode
6. **Incremental**: Git-based incremental analysis works correctly
7. **Documentation**: CI integration guide with working examples

## Benefits

1. **CI/CD Ready**: Drop-in integration with existing CI pipelines
2. **Quality Gates**: Enforce code quality standards automatically
3. **Fast Feedback**: Developers see analysis results in PRs
4. **Incremental**: Only analyze changed files for speed
5. **Flexible**: Multiple output formats for different CI tools
6. **Automated**: No manual intervention required
