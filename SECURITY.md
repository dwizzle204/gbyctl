# Security Policy

## Supported Scope

`gbyctl` is intended for bounded Ubuntu operations workflows. Security issues include:

- policy bypasses
- command injection paths
- privilege escalation beyond intended `sudo` flows
- plaintext exposure of protected state or secrets
- incorrect classification that causes unsafe execution
- repository or release-process weaknesses that could affect users

## Reporting a Vulnerability

Do not open public issues for suspected vulnerabilities.

Use one of these private channels:

- GitHub private vulnerability reporting for this repository, if enabled
- GitHub private contact with the repository owner

Include:

- a clear description of the issue
- affected files or commands
- proof-of-concept steps if safe to share
- potential impact
- any suggested mitigation

## Response Expectations

The goal is to:

- acknowledge a valid report quickly
- reproduce and assess impact
- fix the issue before public disclosure when reasonable
- document any user-facing mitigation or upgrade guidance

## Security Principles

This repository treats these as non-negotiable:

- least privilege
- verify before modify
- explicit approval for risk
- preserve auditability
- prefer reversible changes
- do not weaken authentication or network controls casually
