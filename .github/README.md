# GitHub Automation for AI Media Engine

This directory contains GitHub-specific configuration files.

## What's here?

| File | Purpose |
|------|---------|
| `PULL_REQUEST_TEMPLATE.md` | Shown when opening a PR |
| `README.md` | This file (you're reading it) |
| `ISSUE_TEMPLATE/` | Templates for bugs/features (if you add them) |
| `workflows/` | CI/CD pipelines (if you add them) |

## Recommended additions (not yet created)

### `.github/ISSUE_TEMPLATE/bug_report.md`

```markdown
---
name: Bug report
about: Something isn't working
title: '[BUG] '
labels: bug
---

**Describe the bug**
...

**To reproduce**
1. Run `...`
2. See error

**Expected behavior**
...

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: `rustc --version`
- AIMF version: `aimf --version` or commit hash

**Additional context**
...