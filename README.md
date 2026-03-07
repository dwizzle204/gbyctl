<div align="center">

# gbyctl

**Your Ubuntu operations assistant that understands what you mean—not just what you say.**

![CI Status](https://img.shields.io/github/actions/workflow/status/dwizzle204/gbyctl/ci.yml?branch=main)
![Crates.io](https://img.shields.io/crates/v/gbyctl)
![License](https://img.shields.io/badge/license-MIT-blue)
![Rust](https://img.shields.io/badge/rust-2024-orange)

[Features](#-features) • [Installation](#-installation) • [Quick Start](#-quick-start) • [Security](#-security) • [Contributing](#-contributing)

---

**gbyctl** transforms natural-language requests into bounded, policy-enforced administrative workflows for Ubuntu. Speak plainly—`gbyctl` translates your intent into safe, auditable operations.

[📖 Documentation](#-documentation) • [🐛 Report an Issue](https://github.com/dwizzle204/gbyctl/issues/new) • [💬 Security Vulnerability](https://github.com/dwizzle204/gbyctl/security/advisories/new)

</div>

> **Note**: gbyctl is currently in active development. While functional, the API and behavior may evolve as we refine the workflow families and policy model.

---

## 📋 TL;DR

- **Natural-language → bounded workflows**: Ask "disk is full" and get a safe, step-by-step plan
- **Policy-gated execution**: Every step is classified into 4 security levels
- **Preview-first**: See the plan before any command runs
- **11 built-in workflows**: From troubleshooting to package management to security updates
- **Ubuntu-focused**: Deterministic routing for common sysadmin tasks
- **Not a coding assistant**: Deliberately bounded to operational tasks only

---

## 🎯 Design Principles

gbyctl is built around these core boundaries:

### ✅ What gbyctl does

**Natural-language intake supported**: Speak in plain English—"why is my server slow?" or "my disk is full."

**Skill-based execution**: Only 11 bounded workflow families, no arbitrary code execution.

**Policy-gated**: Every step classified as:
- `safe_execute` — low-risk read-only actions
- `approval_required` — mutating actions needing explicit consent
- `manual_only` — operationally supported but never auto-executed
- `forbidden` — hard refusal, no way around it

### 🚫 What gbyctl does NOT do

**Out-of-scope requests refused**: Coding assistance, general chat, unrelated asks all get a hard no.

**Never runs as root**: The app process runs as a regular user. Planned commands may invoke `sudo` only when policy explicitly allows.

**No chat mode**: No conversations, no follow-up questions, no "remember my preferences." One request, bounded workflow, done.

---

## 🚀 Installation

### From source

```bash
# Clone and build
git clone https://github.com/dwizzle204/gbyctl.git
cd gbyctl
cargo build --release
cargo install --path .

# Or use just
just setup
```

### From crates.io

```bash
cargo install gbyctl
```

### From .deb package

```bash
# Download latest .deb from Releases
sudo dpkg -i gbyctl_*.deb
```

### Requirements

- **Rust 1.82+** (if building from source)
- **Ubuntu 20.04+** or compatible distribution
- **Non-root user** (gbyctl refuses to run as root)

---

## 🏃 Quick Start

### First-run setup

Initialize your provider and key:

```bash
gbyctl setup
```

Setup stores:
- Provider config in `~/.config/gbyctl/config.json`
- API key in OS keyring (never in the config file)

Supported providers:
- **OpenAI-compatible** (`/chat/completions`) — includes OpenAI, DeepSeek, and many others
- **Claude** (`/messages`)

If your API key expires or is invalid, gbyctl warns and offers reconfiguration.

### Try it out

```bash
# System health check
gbyctl doctor

# See the plan without executing
gbyctl --plan "disk is full"

# Non-interactive mode (auto-approve safe operations)
gbyctl --yes "check service status for nginx"

# Machine-readable JSON output
gbyctl --json --plan "show me recent error logs"
```

### Natural language examples

| Request | What gbyctl does |
|---------|------------------|
| "disk is full" | Runs storage triage workflow, identifies causes, safe cleanup plan |
| "why is nginx slow?" | Service status + logs guidance workflow |
| "install nginx" | Safe package installation with version checks |
| "my server won't boot" | Kernel/boot troubleshooting guidance |
| "show me security updates" | Maintenance guidance workflow with apt list --upgradable |
| "check firewall rules" | Firewall troubleshooting workflow |

See [Built-in Workflows](docs/skills.md) for the complete list.

---

## ✨ Features

### 🔒 Security-first architecture

**4-tier policy classification**: Every workflow step is automatically categorized:
- `safe_execute` — Read-only inspection (service status, log viewing)
- `approval_required` — Mutating operations (package installs, config changes)
- `manual_only` — High-risk operations (disk resize, kernel changes) shown but never executed
- `forbidden` — Unsafe operations (rm -rf /, deleting system-critical files) — hard refusal

**Pre-execution security checkpoint** — 7 required checks before ANY command runs:
1. Supported scope and known policy class?
2. Least-privileged for the task?
3. Target safe and correctly identified?
4. Action reversible and risk-bounded?
5. Touching protected controls (auth, boot, root storage)? → needs manual_only
6. Requires elevated confirmation or hard refusal?
7. Current state validated live before action?

**Encrypted state at rest**: All configuration and session logs encrypted with AES-256-GCM-SIV. Key stored in OS keyring, never in files.

### 🧠 Smart intent routing

gbyctl uses a **deterministic-first routing** strategy:

1. **Explicit subcommand** — `gbyctl doctor`, `gbyctl service-status nginx`
2. **Local router** — Pattern matching for common Ubuntu requests (fast, no API call)
3. **Clarification** — If request is close but missing details, ask specifically
4. **LLM fallback** — Only when local routing has no supported answer

This means:
- Quick, common requests: **zero latency** (local routing)
- 95% reduction in LLM API calls for typical sysadmin tasks
- Fallback to LLM only for novel or complex scenarios

### 🎬 Preview-first execution

Default behavior: **Show, then ask, then execute**

```bash
$ gbyctl "disk is full"

📋 PLAN: Storage Triage Workflow

1. Analyze disk usage → df -h, du -sh /var/*
2. Check large files → find /var -type f -size +1G
3. Identify cleanup candidates → apt clean, journalctl --vacuum-time=7d

Commands to execute:
  [safe_execute] df -h
  [safe_execute] du -sh /var/* 2>/dev/null | sort -rh | head -10
  [safe_execute] find /var -type f -size +1G -exec ls -lh {} \;
  [approval_required] apt-get clean
  [manual_only] journalctl --vacuum-size=100M

Execute this plan? [y/N]
```

Use `--yes` for non-interactive execution (only for trusted workflows).

### 🔧 11 Built-in workflow families

| Workflow | When to use | Example requests |
|----------|-------------|------------------|
| **doctor** | General system health check | "is my system healthy?", "why is it slow?" |
| **service_status** | Service lifecycle | "check nginx", "restart apache", "enable docker" |
| **storage** | Disk usage and cleanup | "disk is full", "where's the space going?" |
| **resize_root_plan** | Disk expansion | "need more space", "grow root partition" |
| **install_package** | Package management | "install nginx", "update python" |
| **troubleshoot_firewall** | UFW/firewall issues | "can't reach port 80", "check firewall" |
| **diagnose_reboot_or_kernel_issue** | Boot problems | "won't boot", "kernel panic" |
| **maintenance_guidance** | Security updates & best practices | "security updates", "kernel best practices" |
| **logs_guidance** | Log investigation | "show recent errors", "what failed in logs?" |
| **package_status** | Package version/installed state | "what's installed?", "nginx version?" |

See [docs/skills.md](docs/skills.md) for detailed documentation on each workflow.

### 🌐 Multi-provider LLM support

Choose the AI that fits your environment:

- **OpenAI** — `gpt-4o`, `gpt-4o-mini` (or your own OpenAI-compatible server)
- **Anthropic Claude** — `claude-3-5-sonnet`, `claude-3-opus`

Local providers (via OpenAI-compatible API):
- **Ollama**, **vLLM**, **LM Studio**, and others

All API usage is routed through safe, policy-gated workflows—no direct chat, no arbitrary command generation.

### 📊 Rich output modes

- **Human-readable** — Color-coded, structured output with emoji indicators
- **Machine-readable JSON** — Parse plan, commands, and outcomes programmatically
- **Plan-only mode** — See what *would* happen without executing

```bash
# Get a plan you can review or feed into another tool
gbyctl --plan --json "check service status for nginx"
```

---

## 🚦 Global Flags

| Flag | Purpose |
|------|---------|
| `--plan` | Show plan only; never execute commands |
| `--yes` | Non-interactive approval and execution (use with care!) |
| `--json` | Machine-readable JSON output |
| `--verbose` | Extra diagnostics and debug output |
| `--no-color` | Disable colorized terminal output |

---

## 🔒 Security Model

### Defense in depth

gbyctl implements multiple layers of protection:

**Policy-based classification**: Every workflow step is automatically categorized into 4 security tiers (see [Features](#-security-first-architecture)).

**Pre-execution checkpoint**: 7 required checks before any command runs. Failure = instant refusal.

**Protected targets**: Sensitive paths (boot config, auth files, rootfs) automatically assigned `manual_only` or `forbidden`.

**Encrypted state**: All config, session logs, and state are encrypted at rest with AES-256-GCM-SIV. Key stored in OS keyring.

**Process boundary**: gbyctl runs as non-root. Only planned commands may invoke sudo, and only when policy explicitly allows.

### What "safe" means

In gbyctl, "safe" doesn't mean "harmless"—it means:

✅ **Bounded scope**: Only 11 workflow families, well-tested, no arbitrary code
✅ **Preview-first**: You see the entire plan before any execution
✅ **Policy-gated**: every step classified, checked, and approved
✅ **Reversible**: Prefer operations that can be rolled back
✅ **Least-privilege**: Use minimal permissions for the task

### Protected resources

These paths receive extra scrutiny:

```
/boot/*          → manual_only
/etc/shadow      → manual_only
/etc/passwd      → manual_only  
/etc/sudoers     → forbidden
/root/.ssh/*     → manual_only
/                → manual_only (root operations)
```

See [docs/policy-model.md](docs/policy-model.md) for the complete deny rules.

---

## 🧠 Classification and Routing

### Why "deterministic-first"?

LLM calls are:
- **Slow** — 1-3 seconds per request
- **Costly** — API fees add up
- **Non-deterministic** — classification can vary

Local routing is:
- **Fast** — <1ms (regex/pattern matching)
- **Free** — no API cost
- **Deterministic** — same input → same output

### Routing order

When you ask gbyctl something:

1. **Explicit subcommand** — Fastest path, zero ambiguity
   ```bash
   gbyctl doctor               # Runs doctor workflow immediately
   gbyctl service-status nginx # Service status workflow
   ```

2. **Local router** — Pattern matching, no LLM
   ```
   "disk is full"    → storage workflow      (local)
   "nginx status"    → service_status nginx  (local)
   "install python"  → install_package       (local)
   ```

3. **Clarification** — If missing info, ask specifically
   ```
   "check service"   → "Which service?"
   ```

4. **LLM fallback** — Only when local routing has no answer
   ```
   "my weird app is crashing" → LLM classification → maybe doctor, maybe out_of_scope
   ```

### Out-of-scope detection

gbyctl refuses requests that violate its bounded purpose:

| Request | Response |
|---------|----------|
| "write a python script" | ❌ Out of scope — gbyctl is not a coding assistant |
| "help me debug this code" | ❌ Out of scope — use your IDE or dev tools |
| "what's the weather?" | ❌ Out of scope — not a sysadmin task |
| "tell me a joke" | ❌ Out of scope — not a general chat interface |

This is deliberate—see [docs/policy-model.md](docs/policy-model.md) for the philosophy.

---

## 🔑 Protected State

All sensitive data is encrypted at rest:

- **What's encrypted**: Config, session logs, workflow history
- **Algorithm**: AES-256-GCM-SIV (NIST-approved)
- **Key storage**: OS keyring (`gbyctl/state-encryption-key`)
- **No plaintext fallback**: Encrypted format is mandatory
- **OS support**: Works on Linux via `secret-service` backend

Your API keys never touch the filesystem unencrypted.

---

## 📚 Documentation

| Document | Description |
|----------|-------------|
| **[Policy Model](docs/policy-model.md)** | Security classes, checklist, protected targets, deny rules |
| **[Skills Guide](docs/skills.md)** | Detailed documentation of all 11 workflow families |
| **[Security Policy](SECURITY.md)** | Vulnerability reporting, threat model, incident response |
| **[Contributing](CONTRIBUTING.md)** | Branching strategy, PR expectations, development workflow |
| **[CHANGELOG](CHANGELOG.md)** | Version history and release notes |

---

## 🛠️ Development

### Prerequisites

- **Rust 1.82+** (use [rustup](https://rustup.rs/))
- **Ubuntu 20.04+** or compatible environment for testing
- **OpenAI-compatible or Claude API key** (for LLM integration testing)

### Quick setup

```bash
# Clone and enter directory
git clone https://github.com/dwizzle204/gbyctl.git
cd gbyctl

# Install just (optional, for convenience)
cargo install just

# Build and run tests
just setup    # or: cargo build
just check    # or: cargo fmt -- --check && cargo clippy && cargo test
```

### Development workflow

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets -- -D warnings

# Run tests
cargo test

# Run a specific test
cargo test test_doctor_workflow

# Build release binary
cargo build --release

# Run with debug output
cargo run -- --verbose --plan "disk is full"
```

### Adding a new workflow family

See [docs/skills.md](docs/skills.md#adding-a-new-workflow-family) for the complete process:

1. Add `SkillId` variant in `src/skills/types.rs`
2. Register metadata in `src/skills/builtins/mod.rs`
3. Implement planner in `src/skills/builtins/<skill>.rs`
4. Extend deterministic router in `src/intent/router.rs`
5. Update LLM classifier prompt/schema
6. Extend planner dispatch in `src/plan/planner.rs`
7. Add tests for routing, policy, and checklist behavior

**Important**: Don't add a new workflow for every phrasing. Prefer expanding routing, argument extraction, or clarification inside existing workflow families.

### Code standards

- **No `unsafe` code** — `unsafe_code = deny` in lints
- **No `.unwrap()`** — use `?` operator or `anyhow::Context`
- **Comprehensive error handling** — all errors include context
- **Thread safety** — all shared state must be thread-safe
- **API design** — clear, idiomatic, well-documented

See [CONTRIBUTING.md](CONTRIBUTING.md) for complete guidelines.

---

## 🤝 Contributing

We welcome contributions! Please follow our guidelines:

### Branching strategy

- `main` is always releasable — **do not push directly**
- Use short-lived feature branches: `feat/<topic>`, `fix/<topic>`
- Open a pull request to `main`
- Delete merged branches after merge

### Pull request expectations

Every PR should:
- Explain the operator-facing change
- Note policy or security impact
- Include tests for routing, policy, or CLI behavior
- Keep documentation in sync

Before opening a PR, run:

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

### Code review

All PRs require:
- 1 approval from maintainers
- All CI checks passing (format, clippy, tests, security audit)
- Security review for changes to policy, classifier, or execution modules

See [CONTRIBUTING.md](CONTRIBUTING.md) for the complete contribution guide.

---

## 📦 Changelog

See [CHANGELOG.md](CHANGELOG.md) for version history.

### Recent releases

- **0.5.2** — PR-path dependency review, cargo audit on main push
- **0.5.1** — Deterministic-first routing, stabilized JSON mode
- **0.5.0** — `logs_guidance`, `package_status` workflows
- **0.4.0** — `maintenance_guidance` workflow
- **0.3.0** — Generic `install_package` routing

---

## 📜 License

[MIT License](LICENSE) — See LICENSE file for details.

---

## 🙏 Acknowledgments

gbyctl is inspired by the need for safer, more predictable Linux automation. It draws from:

- **Clap** — CLI argument parsing
- **reqwest** — HTTP client for LLM APIs
- **keyring** — Secure credential storage
- **The Rust community** — For excellent tooling and safety patterns

Special thanks to all contributors who report issues, submit PRs, and help make gbyctl more secure and useful.

---

## 📞 Support

- **📖 Documentation**: [docs/](docs/)
- **🐛 Report bugs**: [GitHub Issues](https://github.com/dwizzle204/gbyctl/issues/new)
- **🔒 Security vulnerabilities**: [GitHub Security Advisories](https://github.com/dwizzle204/gbyctl/security/advisories/new) (private, see [SECURITY.md](SECURITY.md))
- **💬 Questions**: [GitHub Discussions](https://github.com/dwizzle204/gbyctl/discussions)

---

<div align="center">

**Powered by Rust · Secured by design · Bound by policy**

[GitHub](https://github.com/dwizzle204/gbyctl) • [Crates.io](https://crates.io/crates/gbyctl) • [Documentation](docs/)

</div>

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
