# Contributing

## Operating Model

`gbyctl` is a bounded Ubuntu operations assistant. Contributions must preserve these constraints:

- natural-language intake is allowed
- execution stays inside bounded workflows
- policy gates are mandatory
- out-of-scope requests must remain refused
- the tool must not drift into chat, coding, or arbitrary shell automation

## Contribution Path

This repository prefers a fork-and-pull-request model.

Why:

- keeps write access to the canonical repository tightly limited
- reduces the chance of accidental direct pushes or branch sprawl in the main repo
- aligns with a public, security-sensitive project that has a single maintainer

Use this model unless you are the repository owner or trusted automation already operating inside the main repository.

Recommended flow for contributors:

1. Fork `dwizzle204/gbyctl`.
2. Create a short-lived topic branch in your fork.
3. Push your branch to your fork.
4. Open a pull request from your fork into `dwizzle204/gbyctl:main`.

Maintainer exceptions:

- the repository owner may use short-lived branches in the main repository when appropriate
- Dependabot and GitHub-managed automation may open branches directly in the main repository

## Branching Strategy

`main` is protected and should remain releasable.

Rules:

- do not push directly to `main`
- prefer one logical change per pull request
- delete merged branches after merge
- use squash merge for accepted pull requests

Recommended branch names:

- `feat/<short-topic>`
- `fix/<short-topic>`
- `docs/<short-topic>`
- `chore/<short-topic>`
- `refactor/<short-topic>`
- `test/<short-topic>`

Examples:

- `feat/package-status-routing`
- `fix/json-output-contract`
- `docs/fork-contribution-guide`

## Local Setup

```bash
git clone https://github.com/<your-user>/gbyctl.git
cd gbyctl
git remote add upstream https://github.com/dwizzle204/gbyctl.git
cargo build
```

Keep your fork current:

```bash
git fetch upstream
git checkout main
git merge --ff-only upstream/main
git push origin main
```

## Pull Request Expectations

Every pull request should:

- explain the operator-facing or maintainer-facing change
- note any policy or security impact
- include tests for routing, policy, CLI, or workflow behavior where applicable
- keep docs, changelog, and versioning in sync when contracts change
- avoid unrelated refactors in the same branch

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## CI And QA

GitHub Actions is expected to validate:

- formatting
- clippy with warnings denied
- test suite
- release build viability
- dependency review on pull requests
- code coverage artifact generation
- security analysis and dependency auditing

A green pull request should mean the branch is mechanically safe to review, not just syntactically valid.

## Review Standard

Pull requests should make these things easy to verify:

- what changed
- why it changed
- how it was validated
- what the risk is
- how to roll it back

If a change affects sudo use, auth, firewall, boot, storage, package mutation, encryption, or policy enforcement, the PR description should say so explicitly.

## Security And Safety

Contributors must not weaken:

- least-privilege execution
- approval requirements for mutating commands
- manual-only boundaries for high-risk operations
- refusal behavior for out-of-scope or unsafe requests
- encrypted local state handling
- auditability of plans and outcomes

Report vulnerabilities through the private path documented in [SECURITY.md](SECURITY.md), not in public issues.
