# gbyctl

`gbyctl` is an Ubuntu-focused Linux operations assistant that converts explicit or natural-language requests into bounded, policy-enforced administrative workflows.

## Design boundaries

- Natural-language intake is supported.
- Execution is always skill-based and policy-gated.
- Out-of-scope requests (coding/general chat/unrelated asks) are refused.
- `gbyctl` must not be started as root.

## Quick start

```bash
cargo build
cargo run -- setup
cargo run -- --plan doctor
cargo run -- "disk is full"
```

## First-run model setup

Run:

```bash
gbyctl setup
```

Setup stores:

- Provider config in `~/.config/gbyctl/config.json`
- API key in OS keyring (not in config file)

Supported providers:

- OpenAI-compatible (`/chat/completions`)
- Claude (`/messages`)

If an API key is expired/invalid, `gbyctl` warns and offers reconfiguration.

## Global flags

- `--plan`: show plan only; never execute
- `--yes`: non-interactive approval and execution
- `--json`: machine-readable output
- `--verbose`: extra diagnostics
- `--no-color`: disable colorized output

## Execution model

Default behavior is preview-first:

1. Build and show plan
2. Show commands and policy classes
3. Prompt: `Execute this plan now? [y/N]`

Non-interactive runs require `--yes` for execution.

## Classification model

Common supported Ubuntu requests are resolved locally first.

Resolution order:

1. explicit subcommand
2. deterministic local router
3. local clarification for missing bounded fields
4. LLM fallback only when local routing has no supported answer

This keeps common requests lightweight and reduces dependency on provider calls for known workflows.

## Security model

- Every step is classified as: `safe_execute`, `approval_required`, `manual_only`, or `forbidden`.
- A compact pre-execution security checklist is enforced for every step.
- Any checklist failure stops execution with refusal.
- Commands may use `sudo` when needed, but the app process itself must run as a regular user.

## Protected local state

State and session logs are encrypted at rest.

- Encryption: `AES-256-GCM-SIV`
- Encryption key: OS keyring (`gbyctl/state-encryption-key`)
- State plaintext fallback is disabled (encrypted format required)

## Repository governance

The repository is intended to use a protected public `main` branch with short-lived topic branches and pull requests.

Local governance files included in this repo:

- `CONTRIBUTING.md`
- `SECURITY.md`
- `.github/CODEOWNERS`
- `.github/PULL_REQUEST_TEMPLATE.md`
- minimal `.github/workflows/ci.yml`
- minimal `.github/workflows/security.yml`

The root [`.gitignore`](/home/dm/workspace/gbyctl/.gitignore) also excludes common secret-bearing files and local state/config directories to reduce accidental secret commits.

## Workflow Families

`gbyctl` is built around stable Ubuntu operations workflows rather than a large list
of app-specific recipes. Natural-language requests are mapped into these bounded
families:

- `doctor`: host health, slow-server triage, and service overview
- `service_status`: diagnose a specific service
- `disk_full_triage`: investigate disk pressure and cleanup candidates
- `inspect_storage`: inspect block devices, mounts, and filesystem layout
- `resize_root_plan`: manual-only root volume growth planning
- `install_package`: install an Ubuntu package via `apt` after approval
- `troubleshoot_firewall`: inspect listeners and firewall state, optionally open a port
- `diagnose_reboot_or_kernel_issue`: inspect reboot and kernel evidence
- `maintenance_guidance`: review evergreen security, updates, and kernel practices
- `logs_guidance`: inspect recent system and service logs
- `package_status`: check package install/version/update state

Examples:

```bash
gbyctl "why is my server slow"
gbyctl "disk is full"
gbyctl "show running services"
gbyctl "install nginx"
gbyctl "open port 8080"
gbyctl "why did my server reboot"
gbyctl "what are the security best practices for updates and kernels"
gbyctl "show me recent logs for nginx"
gbyctl "what version of nginx is installed"
```
