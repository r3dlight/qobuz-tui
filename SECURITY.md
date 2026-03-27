# Security Policy

## Reporting a Vulnerability

If you discover a security vulnerability in this project, please report it responsibly.

Please report security issues through one of these channels:

- **GitHub Issue:** [Open an issue](https://github.com/r3dlight/qobuz-tui/issues/new) with the `security` label
- **Private advisory:** [GitHub Security Advisories](https://github.com/r3dlight/qobuz-tui/security/advisories/new) (for sensitive issues you prefer not to disclose publicly)
- **Maintainer:** [@r3dlight](https://github.com/r3dlight)

## What to include

- Description of the vulnerability
- Steps to reproduce
- Potential impact
- Suggested fix (if any)

## Response time

I will acknowledge your report within 48 hours and aim to provide a fix or mitigation within 7 days for critical issues.

## Scope

This policy covers:

- The `qobuz-lib` and `qobuz-tui` Rust crates
- Authentication and credential handling
- The Landlock sandbox implementation
- Audio streaming and caching logic

Out of scope:

- The Qobuz API itself (report to Qobuz directly)
- Vulnerabilities in third-party dependencies (report upstream, but feel free to notify me)
