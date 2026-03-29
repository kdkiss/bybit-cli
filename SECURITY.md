# Security Policy

## Supported Versions

Security fixes are applied to the latest release on the default branch.

## Reporting A Vulnerability

Do not open a public issue for a suspected vulnerability.

Preferred reporting paths:
- use GitHub private vulnerability reporting if it is enabled for this repository
- otherwise contact the maintainer privately and include enough detail to reproduce the issue safely

When reporting, include:
- affected command, file, or workflow
- impact and attack scenario
- reproduction steps
- whether secrets, credentials, signing keys, or release artifacts are involved

Please do not include real API keys, secrets, or private signing material in any report.

## Response Expectations

The goal is to acknowledge reports promptly, validate impact, fix confirmed issues, and publish a coordinated release when appropriate.

## Scope Notes

Security-sensitive areas in this repository include:
- API credential handling
- signing and release automation
- MCP tool exposure and dangerous tool gating
- workflow and supply-chain configuration
