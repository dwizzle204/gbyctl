# Policy Model

## Policy classes

- `safe_execute`: low-risk read-only actions
- `approval_required`: mutating actions requiring explicit approval
- `manual_only`: action is operationally supported but never auto-executed
- `forbidden`: hard refusal, no operationalization

## Compact security checklist

Before each step executes, `gbyctl` enforces a required checklist:

1. Is this within supported scope and known policy class?
2. Is this least-privileged for the task?
3. Is the target safe and correctly identified?
4. Is the action reversible/risk-bounded?
5. Does it touch protected controls (auth, boot, root storage, core security)?
6. Does it require elevated confirmation or hard refusal?
7. Was current state validated live before action?

If any check fails, execution stops with refusal.

## Protected targets and deny rules

`manual_only` and `forbidden` boundaries are implemented in:

- `src/policy/deny.rs`
- `src/policy/classifier.rs`
- `src/policy/checklist.rs`

Examples:

- `sudo apt-get install ...` -> `approval_required`
- `growpart ...` -> `manual_only`
- `rm -rf /` -> `forbidden`

## Root process policy

`gbyctl` refuses startup when run as root.

- App must run as non-root user.
- Planned commands may still invoke `sudo` when policy allows.

## Plan preview policy

Execution is preview-first by default:

- plan shown first
- command list shown before execution
- explicit `Execute? [y/N]` prompt
- non-interactive execution requires `--yes`
