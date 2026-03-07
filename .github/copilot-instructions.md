# Copilot Instructions for gbyctl

## Project intent
- `gbyctl` is a natural-language Ubuntu operations assistant.
- It maps user requests to bounded OS-admin workflows.
- It is not a general chatbot or coding assistant.

## Hard safety boundaries
- Never run the application process as root.
- Commands may use `sudo` only when policy allows and user approval is explicit.
- Keep deterministic routing first; use LLM fallback only when local routing has no supported match.
- Never bypass policy classes: `safe_execute`, `approval_required`, `manual_only`, `forbidden`.
- Keep execution preview-first unless explicitly approved to execute.

## UX expectations
- Accept free-form user asks and map to known skills/workflows.
- If intent is incomplete, ask focused operational clarification questions.
- Keep output curated and beginner-friendly, but always show the exact command being run.
- Prefer compact summaries plus bounded output snippets over noisy raw output.

## Security expectations
- Enforce least privilege and reversible actions where possible.
- Refuse out-of-scope asks (coding/refactoring/general chat).
- Do not weaken authentication, firewall, or core host protections casually.
- Keep state and secrets protected; do not store API keys in plaintext files.

## Code conventions
- Keep changes small and explicit.
- Add comments only where intent is non-obvious.
- Prefer reuse over duplicate logic.
- Update tests for behavior changes.
- Keep docs/version/changelog aligned when user-visible behavior changes.

## Validation
- Run:
  - `cargo fmt`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- For workflow changes, validate `.github/workflows/*.yml` syntax and trigger scope.

