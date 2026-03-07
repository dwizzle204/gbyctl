# Changelog

## 0.5.5 - 2026-03-07

- fix the CI packaging tarball exclusion so the source archive no longer tries to package its own output
- pin the Docker builder image to a valid Rust `1.94` Bookworm tag
- add direct Debian package smoke testing in Docker before CI packaging passes or a release publishes
- tighten GitHub Actions permissions to job scope where write access is not required
- align Docker smoke scripts with the actual CLI contract by using the global `--plan` entrypoint
- allow ephemeral CI/container runs without a host keyring by disabling persisted state and audit writes when `GBYCTL_EPHEMERAL=1`
- align Docker smoke runtime images with the supported Ubuntu-only host contract

## 0.5.4 - 2026-03-06

- add version-metadata verification in CI and release workflows
- add packaging and Docker smoke-test jobs to validate distributable CLI behavior
- add a tag-driven GitHub release workflow for binary, source, and Debian artifacts
- fix source tarball naming to use `gbyctl-<version>.tar.gz`
- align Docker builder and declared MSRV with the current stable Rust features used by the codebase

## 0.5.3 - 2026-03-06

- strengthen `CONTRIBUTING.md` around a fork-first contribution model for public contributors
- add free CI code coverage artifact generation with `cargo-llvm-cov`
- add PR and main-branch security scanning with CodeQL and cargo-audit

## 0.5.2 - 2026-03-06

- add PR-path dependency review and release-build validation in GitHub Actions CI
- run cargo audit on pushes to `main` in addition to weekly scheduled security scans
- tighten pull request and issue templates for reproducible reports and clearer scope checks

## 0.5.1 - 2026-03-06

### Changed

- Switched natural-language classification to deterministic-first routing with LLM fallback only for unsupported local cases.
- Stabilized JSON mode so `plan-only`, `clarification`, `out_of_scope`, `execute`, and `manual-only` return machine-readable output without mixed human log lines.
- Simplified `.github` to a minimal public-repo governance set with focused CI/security workflows and lightweight issue/PR templates.

### Added

- Added broader routing accuracy matrix tests and CLI JSON contract tests.
- Hardened `.gitignore` to exclude common secret-bearing files and local config/state directories.

## 0.5.0 - 2026-03-06

### Added

- Added `logs_guidance` workflow for recent journal and service log investigation.
- Added `package_status` workflow for install/version/update checks.
- Added explicit `package-status`, `logs`, and `maintenance` CLI paths.

### Changed

- Expanded natural-language routing so package version/install-state asks and log-investigation asks map into bounded operational workflows.

## 0.4.0 - 2026-03-06

### Added

- Added `maintenance_guidance` workflow for evergreen security, updates, and kernel best-practice requests.

### Changed

- Collapsed storage inspection and disk-pressure planning into a shared storage workflow module.
- Expanded routing so best-practice maintenance asks map into a bounded operational workflow instead of requiring new one-off skills.

## 0.3.0 - 2026-03-06

### Changed

- Reframed built-ins as stable Ubuntu workflow families instead of example-specific skills.
- Replaced `install_tomcat` with generic `install_package` routing and planning.
- Expanded natural-language routing for slow-server, running-services, package-install, and broader firewall/service phrasing.
- Enriched `doctor` with memory and running-service inspection so it better serves beginner operators.

### Added

- Explicit `install package <name>` CLI path alongside the existing Tomcat shortcut.
- Package-name sanitization for approved apt installs.

## 0.2.0 - 2026-03-06

### Added

- LLM-backed intent classification with provider support:
  - OpenAI-compatible
  - Claude
- First-run interactive setup (`gbyctl setup`) with persisted provider config.
- Secure key storage via OS keyring for provider API keys.
- Root process startup guard (`gbyctl` refuses to run as root).
- Default preview-first execution flow with explicit `Execute? [y/N]` prompt.
- Compact pre-execution security checklist enforcement.
- Encrypted-at-rest state and session logs using `AES-256-GCM-SIV`.
- Secure keyring-managed state encryption key.

### Changed

- Clarification responses now use `clarification` mode (not `manual-only`).
- Refusal derivation now follows policy/checklist results instead of brittle raw-text special-casing.
- Intent construction in CLI was deduplicated for maintainability.

### Security

- Plaintext state fallback was removed; encrypted state format is required.
- Manual-only/forbidden safeguards are enforced consistently before command execution.

## 0.1.0 - 2026-03-06

### Added

- Initial Rust implementation of `gbyctl` with bounded skill model.
- Core policy classes (`safe_execute`, `approval_required`, `manual_only`, `forbidden`).
- Starter skills and command execution scaffolding.
- Basic state cache and session audit logging.
