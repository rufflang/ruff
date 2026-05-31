# Reserved Name Dogfood Notes

## What was easy

- Adding a single source of truth (`config/reserved_names.toml`) and loading it centrally.
- Enforcing reserved namespace and alias checks at CLI routing boundaries.
- Adding basic trust-mode validation in manifest/package parsing.

## What was awkward

- Current workflow-pack architecture was namespace-first (`ruff <namespace> <command>`), so introducing reserved workflow families required contribution metadata instead of direct command ownership.
- Built-in pack discovery/registration paths needed cleanup to avoid duplicate registration paths.

## Missing core capabilities

- No first-party `ruff doctor` command execution path yet for contributed profiles.
- No signature-based trust root for first-party external artifacts.

## Missing manifest capabilities

- Manifest trust/source is inferred by loader path, not by cryptographic identity.
- Contribution schema is currently minimal (`doctor_profiles`) and not yet consumed by a core `doctor` dispatcher.

## Missing package-manager capabilities

- No Kennel registry uniqueness API yet.
- No scope ownership model (`@user`/`@org`) in CLI/package parsing.
- No publish-time registry validation for reserved names beyond local checks.

## Recommended follow-up work

- Implement first-party `ruff doctor` with `--profile`, positional profile, and `--list-profiles` using `contributes.doctor_profiles`.
- Add signed first-party metadata and trust-root verification.
- Add explicit Kennel package/scope validators and server-side reserved-name enforcement.
- Add integration tests for `ruff pack run` and future `ruff doctor` profile dispatch.

