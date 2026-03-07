# Security Policy

This document outlines **gbyctl**'s security policies, threat model, and response procedures.

## Security Principles

1. **User Safety First:** Operations must be safe first, convenient second
2. **Defense in Depth:** Multiple security layers (classification, validation, execution)
3. **Least Privilege:** Never run as root, minimal permissions only
4. **Explicit Approval:** Destructive operations require user confirmation
5. **Fail Secure:** On error, abort rather than proceed unsafely
6. **Transparency**: Clear audit logging of all operations

## Threat Model

### Malicious User Input

**Threat:** User tries to execute harmful commands through command injection.

**Mitigations:**
- Input validation and sanitization (`src/exec/runner.rs`)
- Shell metacharacter detection
- Command chaining prevention
- Policy classification before execution

### Privilege Escalation

**Threat:** User attempts to gain root privileges.

**Mitigations:**
- Root guard prevents running as root (`src/cli/mod.rs`)
- Sudo commands require explicit approval
- Protected path modification blocked
- UID/GID checks before privileged operations

### Protected Resource Access

**Threat:** User attempts to modify critical system files.

**Mitigations:**
- Protected path detection (`src/policy/rules.rs`)
- Deny patterns for destructive commands
- Pre-execution checklist verification
- Write-only filesystem checks

### Policy Bypass

**Threat:** Attacker attempts to bypass security checks.

**Mitigations:**
- Layered policy enforcement
- Case-insensitive regex patterns
- Command chaining detection
- Strict validation at each layer

## Security Architecture

```
User Request
    |
    v
Input Validation
    |
    v
Intent Classification
    |
    v
Policy Classification (4 layers)
    |
    v
Pre-Execution Checklist
    |
    v
User Approval (if required)
    |
    v
Safe Command Execution
    |
    v
Audit Logging
```

### Policy Classes

| Class | Description | Approval Required | Example Commands |
|-------|-------------|-------------------|------------------|
| **SafeExecute** | Non-destructive operations | No | `df -h`, `systemctl status` |
| **ApprovalRequired** | Destructive but manageable | Yes | `systemctl restart`, `apt update` |
| **ManualOnly** | Requires human verification | Manual only | `fdisk`, `reboot` |
| **Forbidden** | Never allowed | No execution | `rm -rf /`, `mkfs` |

### Protected Paths

```rust
pub const PROTECTED_PATHS: [&str; 6] = [
    "/",              // Root filesystem
    "/boot",           # Boot configuration
    "/etc",            # System configuration
    "/usr",            # System binaries
    "/root",           # Root home directory
    "/var/lib",        # System libraries/state
];
```

Writing to these paths requires:
- **ManualOnly** policy (not auto-executed even with approval)
- Explicit user confirmation
- Clear warning about system impact

### Pre-Execution Checklist

Every command execution step must satisfy:

1. **Classification:** Command is classified (not Forbidden)
2. **Read-Only Check:** Write operations have explicit approval
3. **Protected Path:** No writes to protected paths
4. **State Consistency:** State file is not corrupted
5. **Ubuntu Only:** System is running supported Ubuntu version
6. **No Root:** Not running as root user

## Secure Development Practices

### Code Review Requirements

All PRs modifying security-sensitive code require:
- [ ] Security-themed review from maintainers
- [ ] Negative tests added for attack vectors
- [ ] Threat model analysis documented
- [ ] Code follows security standards in [`copilot-instructions.md`](../.github/copilot-instructions.md)

### Security-Sensitive Modules

The following modules are **security-critical**:
- `src/policy/` - Classification and denial logic
- `src/exec/` - Command execution and validation
- `src/verify/` - Verification and checklist enforcement
- `src/state/` - Encryption and key management
- `src/cli/mod.rs` - Guard rails (root check, Ubuntu check)

### Input Validation Checklist

For all user input:
- [ ] Shell metacharacters detected and rejected
- [ ] Command chaining prevented
- [ ] Command substitution denied
- [ ] Path traversal validated
- [ ] Unicode normalization applied
- [ ] Length limits enforced

### Dependency Security

- All dependencies security audited via [`cargo audit`](https://github.com/RustSec/advisory-db)
- Dependabot enabled for automatic updates
- Supply chain verified via [`cargo-supply-chain`](https://github.com/EmbarkStudios/cargo-supply-chain)
- Regular security scanning via GitHub CodeQL

## Incident Response

### Severity Levels

| Severity | Response Time | Example |
|-----------|---------------|----------|
| **Critical** | < 24 hours | Remote code execution, command injection |
| **High** | < 48 hours | Privilege escalation, data corruption |
| **Medium** | < 7 days | Policy bypass, minor injection |
| **Low** | < 30 days | Denial of service, minor bugs |

### Reporting a Vulnerability

**DO NOT file public issues for security vulnerabilities.**

If you discover a security vulnerability in gbyctl:

1. **Email:** Send a report to **security@example.com** (replace with real contact)
2. **Include:**
   - Description of the vulnerability
   - Steps to reproduce (or proof-of-concept)
   - Impact assessment
   - Suggested fix (if known)
   - Your contact information for follow-up

3. **What to expect:**
   - **24 hours:** Initial acknowledgment
   - **48 hours:** Triage and severity assessment
   - **7 days:** Patch or workaround plan (for high/critical)
   - **30 days:** Public disclosure (after fix)

### Security Advisory Process

When vulnerabilities are fixed:

1. **Prepare security advisory** privately
2. **Coordinate release** with downstream consumers
3. **Fix vulnerability** on a security branch
4. **Merge and release** with security advisory
5. **Disclose vulnerability** publicly after release

### Security Post-Mortems

After significant security incidents:

1. **Root cause analysis** - Why did this happen?
2. **Impact assessment** - What was affected?
3. **Timeline** - When did it happen and when was it detected?
4. **Fix verification** - Is the fix complete and tested?
5. **Prevention** - How can we prevent this in the future?
6. **Documentation** - Update threat model and tests

## Security Features

### Encrypted State Storage

```rust
// AES-256-GCM-SIV encryption at rest
pub fn encrypt(plaintext: &[u8]) -> Result<Vec<u8>> {
    // Keyring-managed encryption keys
    // Nonce with getrandom (cryptographically secure)
    // Version 1 blob format with AAD
}
```

### Keyring API Key Storage

```rust
// OS keyring for LLM provider API keys
pub fn store_api_key(key_id: &str, key_value: &str) -> Result<()> {
    keyring::Entry::new("gbyctl", key_id)
        .set_secret(key_value.to_string())?;
    Ok(())
}
```

### Root User Guard

```rust
pub fn is_running_as_root() -> bool {
    nix::unistd::Uid::effective().is_root()
}

pub fn dispatch(cli: Cli) -> Result<()> {
    if is_running_as_root() {
        return output(&cli, "refusal", None,
            "gbyctl must not be run as root.");
    }
    // ...
}
```

## Auditing and Logging

### Audit Events

Every operation is logged to `~/.local/share/gbyctl/audit.log`:

```json
{
  "timestamp": "2026-03-06T12:00:00Z",
  "event": "command_execution",
  "command": "systemctl status nginx",
  "policy_class": "SafeExecute",
  "exit_code": 0,
  "duration_ms": 1234
}
```

### Log Integrity

- Tamper-evident log chaining (not yet implemented, see RUST-011)
- Hash verification at startup
- Rotation logs periodically
- Archive logs for review

## Compliance

### Ubuntu-Specific

- **v1 Scope:** Ubuntu LTS (20.04, 22.04, 24.04) only
- **Ubuntu Detection:** Verified before operations
- **Ubuntu Only Warning:** Fails gracefully on non-Ubuntu systems

### Package Distribution

- **Debian packaging:** Via [`cargo-deb`](https://github.com/kornelski/cargo-deb)
- **Rust crate:** Via [crates.io](https://crates.io)
- **GPG Signing:** Future enhancement (see SEC-018)

## Security References

- [Rust Security Advisory Database](https://github.com/RustSec/advisory-db)
- [OWASP Command Injection](https://owasp.org/www-community/attacks/Command_Injection)
- [Ubuntu Security Notices](https://ubuntu.com/security/notices)
- [Code Review Document](../.copilot-tracking/reviews/2026-03-06/gibby-code-review.md)

## Questions?

For security questions or to report vulnerabilities:

- **Email:** security@example.com (update with real contact)
- **GitHub Issues:** Use [🔒 Security Vulnerability](../.github/ISSUE_TEMPLATE/security_vulnerability.md) template for non-critical issues
- **Maintainer Contact:** `@dm`

---

**Remember:** Security is a process, not a product. Vigilance requires continuous effort.
