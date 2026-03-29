# Secure Software Development Lifecycle (SecureSDLC)

This project adheres to a strict SecureSDLC to ensure the safety of user funds and API credentials.

## 1. Governance & Policy
- **Unofficial Status**: This project is community-maintained and not affiliated with Bybit.
- **Dependency Minimalization**: We keep the dependency tree lean to reduce the attack surface.
- **No Persistence of Secrets**: The CLI never logs or transmits API secrets to any destination other than the official Bybit API endpoints.

## 2. Secure Design
- **Local Credentials**: API keys are stored in the user's home directory with restricted file permissions.
- **Paper Trading First**: We provide a full-fidelity paper trading engine to allow strategy testing without financial risk.
- **Atomic Flattening**: Built-in emergency exit commands (`position flatten`) to mitigate algorithmic errors.

## 3. Implementation Security
- **Memory Safety**: Built in Rust to prevent buffer overflows and memory corruption.
- **Input Validation**: Strict schema validation for all user-provided parameters.
- **Dry-Run Mode**: Every trade command supports a `--validate` flag to check the request without submitting.

## 4. Verification & Hardening
- **Automated Linting**: All commits must pass `cargo clippy` with zero warnings.
- **Security Auditing**: We run `cargo audit` in CI to detect vulnerabilities in dependencies.
- **Static Analysis**: Continuous integration (CI) includes advisory scans for common security pitfalls.

## 5. Release Security
- **Signed Artifacts**: All official releases are signed with `minisign`.
- **Reproducible Builds**: We strive for build reproducibility to ensure the binary matches the source code.
- **Provenance Attestation**: Release artifacts include GitHub build provenance attestations.

## 6. Incident Response
- **Bug Bounty**: We encourage responsible disclosure of security vulnerabilities.
- **Credential Rotation**: If a breach is suspected, we provide a `bybit auth reset` command to wipe local credentials instantly.
