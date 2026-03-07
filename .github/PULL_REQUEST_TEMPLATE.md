## Summary

<!-- Describe the change in 2-5 concrete bullets. -->

## Why

<!-- Explain the user-visible or maintainer-visible reason for this change. -->

## Validation

```bash
# Paste the exact commands you ran.
```

## Risk

- Scope: low / medium / high
- User impact:
- Rollback:

## Checklist

- [ ] PR targets `main` from a short-lived branch
- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets -- -D warnings`
- [ ] `cargo test`
- [ ] `cargo build --release`
- [ ] Added or updated tests for changed behavior
- [ ] Updated docs, `CHANGELOG.md`, and `VERSION` when behavior or contract changed
- [ ] Not a direct push to `main`
- [ ] Security impact reviewed and explained when auth, sudo, policy, network, or state handling changed
