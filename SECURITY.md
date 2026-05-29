# Security Policy

## Supported Versions

| Version | Supported   |
|---------|-------------|
| 0.1.x   | ✅ (active) |
| < 0.1   | ❌          |

## Reporting a Vulnerability

**Do not open public issues for security vulnerabilities.**

Instead, email **security@aimf.dev** (or use GitHub private vulnerability reporting if enabled).

You should receive a response within **48 hours**. We'll:

1. Acknowledge receipt
2. Investigate and confirm
3. Release a patch (usually within 7 days)
4. Credit you in the release notes (if you want)

## Cryptographic claims

- **Ed25519 signatures** — standard implementation from `ed25519-dalek` (no custom crypto)
- **Hashing** — SHA-256 from `sha2` crate
- **Randomness** — `getrandom` / `OsRng`

No backdoors, no key escrow, no telemetry.

## Past vulnerabilities

None yet. [See security advisory feed](https://github.com/ai-mf/media-engine/security/advisories)

## Key management recommendations for users

- Store private keys with `chmod 600` (Unix) or encrypted (Windows)
- Never commit private keys to version control
- Rotate keys every 6-12 months for high-value content
- Use environment variables or secret managers in production

## Responsible disclosure

We follow standard coordinated disclosure. Please give us 90 days before public disclosure.