#!/bin/bash
# Complete verification of all project files

set -e

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
RESET='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════════════╗${RESET}"
echo -e "${BLUE}║           AIMF Complete Setup Verification                     ║${RESET}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════════════╝${RESET}"
echo ""

COUNT_TOTAL=0
COUNT_PRESENT=0

check_file() {
    COUNT_TOTAL=$((COUNT_TOTAL + 1))
    if [ -f "$1" ]; then
        echo -e "  ${GREEN}✅${RESET} $1"
        COUNT_PRESENT=$((COUNT_PRESENT + 1))
    else
        echo -e "  ${RED}❌${RESET} $1 ${YELLOW}(missing)${RESET}"
    fi
}

check_dir() {
    COUNT_TOTAL=$((COUNT_TOTAL + 1))
    if [ -d "$1" ]; then
        echo -e "  ${GREEN}✅${RESET} $1/"
        COUNT_PRESENT=$((COUNT_PRESENT + 1))
    else
        echo -e "  ${RED}❌${RESET} $1/ ${YELLOW}(missing)${RESET}"
    fi
}

echo -e "${YELLOW}📁 Root files:${RESET}"
check_file "README.md"
check_file "CONTRIBUTING.md"
check_file "SECURITY.md"
check_file "CHANGELOG.md"
check_file "LICENSE"
check_file "CODE_OF_CONDUCT.md"
check_file "MAINTAINERS.md"
check_file "Makefile"
check_file "Dockerfile"
check_file "docker-compose.yml"
check_file "justfile"
check_file "pre-commit-config.yaml"
check_file ".markdownlint.json"
check_file "rust-toolchain.toml"
check_file "clippy.toml"
check_file "taplo.toml"

echo -e "\n${YELLOW}📁 GitHub files:${RESET}"
check_dir ".github"
check_file ".github/pull_request_template.md"
check_file ".github/README.md"
check_dir ".github/workflows"
check_file ".github/workflows/ci.yml"
check_file ".github/workflows/release.yml"
check_dir ".github/ISSUE_TEMPLATE"
check_file ".github/ISSUE_TEMPLATE/bug_report.md"
check_file ".github/ISSUE_TEMPLATE/feature_request.md"
check_file ".github/ISSUE_TEMPLATE/config.yml"

echo -e "\n${YELLOW}📁 Documentation files:${RESET}"
check_dir "docs"
check_file "docs/README.md"
check_file "docs/API.md"
check_file "docs/ARCHITECTURE.md"
check_file "docs/BEST_PRACTICES.md"
check_file "docs/COMPANIES.md"
check_file "docs/FAQ.md"
check_file "docs/GLOSSARY.md"
check_file "docs/IANA-APPLICATION.md"
check_file "docs/ROADMAP.md"
check_file "docs/SCHEMA.md"
check_file "docs/WORKFLOWS.md"

echo -e "\n${YELLOW}📁 Specification files:${RESET}"
check_dir "spec"
check_file "spec/README.md"

echo -e "\n${YELLOW}📁 Scripts:${RESET}"
check_dir "scripts"
check_file "scripts/validate-docs.sh"
check_file "scripts/verify-complete-setup.sh"

echo -e "\n${YELLOW}📁 Configuration files:${RESET}"
check_dir ".cargo"
check_file "cargo/config.toml"
check_dir ".cargo-husky/hooks"
check_file ".cargo-husky/hooks/pre-commit"

echo -e "\n${YELLOW}📊 Summary:${RESET}"
echo -e "  Files present: ${GREEN}$COUNT_PRESENT${RESET} / $COUNT_TOTAL"
PERCENT=$((COUNT_PRESENT * 100 / COUNT_TOTAL))
if [ $PERCENT -eq 100 ]; then
    echo -e "  Status: ${GREEN}✅ Complete!${RESET}"
    exit 0
else
    echo -e "  Status: ${YELLOW}⚠️  Missing $((COUNT_TOTAL - COUNT_PRESENT)) files${RESET}"
    exit 1
fi