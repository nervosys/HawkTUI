# Security Policy

## Supported Versions

We actively support only the latest release.

| Version | Supported |
| ------- | --------- |
| 1.0.x   | Yes       |
| < 1.0   | No        |

## Reporting a Vulnerability

**Do not open a public issue for security vulnerabilities.**

Instead, please report them via GitHub's private vulnerability reporting:

<https://github.com/nervosys/Louie/security/advisories/new>

Or email: **security@nervosys.ai**

We will acknowledge receipt within 48 hours and aim to publish a fix within
7 days for critical issues.

## Security Audit

A detailed security audit is documented in [`docs/SECURITY-AUDIT.md`](docs/SECURITY-AUDIT.md).

## Hardening Measures

- Input size cap: 1 MB per JSON line
- Rate limiting: 1 000 requests / second
- Subscription limit: 100 concurrent
- Terminal dimension clamping: 1–1024
- Action parameter schema validation before dispatch
- Auth handshake support in agent sessions
