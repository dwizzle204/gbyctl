# Contributing

## Scope

`gbyctl` is a bounded Ubuntu operations assistant. Contributions must preserve these constraints:

- natural-language intake is allowed
- planning and execution stay inside bounded workflows
- policy gates are mandatory
- out-of-scope requests must not become chat or coding behavior

## Branching Strategy

This repository uses a protected `main` branch with short-lived topic branches.

- `main` is always releasable
- do not push directly to `main`
- create a topic branch for every change
- open a pull request to merge into `main`
- prefer one logical change per pull request
- delete merged branches after merge

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
- `docs/repo-governance`

## Pull Request Expectations

Every pull request should:

- explain the operator-facing change
- note any policy or security impact
- include tests for routing, policy, or CLI behavior when applicable
- keep documentation in sync when behavior changes

Before opening a pull request, run:

```bash
cargo fmt --all
cargo clippy --all-targets -- -D warnings
cargo test
```

## Merge Strategy

Use squash merges into `main`.

Why:

- keeps `main` history readable
- avoids noisy merge commits from short-lived branches
- makes it easier to reason about release deltas

## Review and Ownership

This is currently a single-maintainer repository.

- only the repository owner should merge pull requests into `main`
- branch protection should require pull requests for `main`
- direct pushes, branch deletion, and force-pushes to `main` should remain blocked

## Security and Safety

Contributors must not weaken:

- least-privilege execution
- approval requirements for mutating commands
- manual-only boundaries for high-risk operations
- refusal behavior for out-of-scope or unsafe requests
- encrypted local state handling

Security-related changes should be documented clearly in the pull request and in repository docs where relevant.
