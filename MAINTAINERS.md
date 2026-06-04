# Maintainer Guide

This doc is for maintainers of the AI Media Engine project. If you're a contributor, see [CONTRIBUTING.md](CONTRIBUTING.md).

## Current Maintainers

| Name | GitHub | Area |
|------|--------|------|
| AI Media Format Contributors | ai-mf | Overall lead |
| (Open) | - | Core/crypto |
| (Open) | - | Codecs |
| (Open) | - | CLI tools |
| (Open) | - | Documentation |

## Responsibilities

### Release Manager (rotating)
- Tag releases
- Build binaries
- Write release notes
- Update changelog

### Code Reviewers
- Review PRs within 48 hours
- Enforce coding standards
- Check for security issues

### Documentation Steward
- Keep docs up to date
- Review doc PRs
- Manage examples

## Release Process

### Preparing a release

```bash
# 1. Update version in Cargo.toml files
cd /path/to/media-engine
sed -i 's/version = "1.0.0"/version = "0.2.0"/' */Cargo.toml

# 2. Update CHANGELOG.md
# Move [Unreleased] to new version, add date

# 3. Run full test suite
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings

# 4. Commit changes
git commit -am "chore: prepare release v0.2.0"

# 5. Create tag
git tag -a v0.2.0 -m "Release v0.2.0"

# 6. Push
git push origin main
git push origin v0.2.0

# 7. GitHub Actions will build and publish
Post-release

    Update documentation website (if exists)

    Announce on Discord/Twitter

    Close milestone on GitHub

    Create next milestone

PR Merge Criteria
Must have:

    ✅ All CI checks pass

    ✅ At least one approving review (two for core changes)

    ✅ No merge conflicts

    ✅ Tests added for new functionality

    ✅ Documentation updated

Should have:

    📝 Clear commit history (squash if needed)

    🧪 Benchmarks for performance changes

    📦 No unnecessary dependencies

Merge strategy:

    Normal PR → Squash merge

    Multi-commit feature → Rebase merge

    Documentation only → Rebase merge

Security Response

If a security vulnerability is reported:

    Acknowledge within 24 hours

    Reproduce and confirm

    Fix in private branch

    Release patch version

    Announce after release (allow 90 days disclosure)

See SECURITY.md for details.
Moderation Guidelines
Code of Conduct violations:

    First offense: Private warning

    Second offense: Temporary ban (30 days)

    Third offense: Permanent ban

Spam/abuse:

    Immediate ban (no warning)

    Report to GitHub

Infrastructure Access
Required for maintainers:

    GitHub write access to repository

    crates.io publish access (if publishing)

    Docker Hub access (if publishing images)

Requesting access:

Open an issue with [MAINTAINER ACCESS] in title. Requires existing maintainer approval.
Deprecation Policy
Removing a feature:

    Announce deprecation with #[deprecated] attribute

    Keep for one minor version (e.g., v0.2 → v0.3)

    Remove in next major version (v1.0)

Breaking changes:

    Only allowed in major versions (v0.x → v1.0, v1.0 → v2.0)

    Must be documented in migration guide

Communication Channels
Channel	Purpose
GitHub Issues	Bug reports, feature requests
GitHub Discussions	Q&A, proposals
#aimf-maintainers (private)	Security, sensitive topics
Monthly sync (calendar invite)	Planning, roadmap
Onboarding New Maintainers

Process:

    Contributor shows sustained activity (3+ months)

    Existing maintainer nominates

    Vote (requires 2/3 approval)

    Add to MAINTAINERS.md and GitHub team

Code Ownership
Path	Owner	Review required
aimf_core/	Core team	2 approvals
codecs/	Codec team	1 approval
tools/	CLI team	1 approval
docs/	Anyone	1 approval
examples/	Anyone	1 approval
Emergency Procedures
CI broken on main:

    Immediate revert (if within 1 hour)

    Otherwise, fix forward

    Notify #aimf-maintainers

Security vulnerability:

    Private communication only

    Prepare patch

    Coordinate release

Critical bug in production:

    Patch release within 24 hours

    Backport to previous version if needed

    Update documentation

Thank you for maintaining AI Media Engine! 🎬