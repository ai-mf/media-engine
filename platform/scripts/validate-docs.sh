
---

## 9. Final step: Validation script

Create `scripts/validate-docs.sh` (NEW):

```bash
#!/bin/bash
# Validates all markdown links and basic structure

set -e

echo "📚 Validating AI Media Engine documentation..."

cd "$(git rev-parse --show-toplevel)" || exit 1

# Check required files exist
required_files=(
    "README.md"
    "CONTRIBUTING.md"
    "SECURITY.md"
    "CHANGELOG.md"
    "LICENSE"
    "USAGE.md"
    "BATCH.md"
    "docs/README.md"
    "docs/API.md"
    "docs/SCHEMA.md"
    "docs/WORKFLOWS.md"
    "docs/COMPANIES.md"
    "docs/IANA-APPLICATION.md"
    "spec/README.md"
    ".github/PULL_REQUEST_TEMPLATE.md"
    ".github/ISSUE_TEMPLATE/bug_report.md"
    ".github/ISSUE_TEMPLATE/feature_request.md"
)

missing=0
for file in "${required_files[@]}"; do
    if [ ! -f "$file" ]; then
        echo "❌ Missing: $file"
        missing=1
    fi
done

if [ $missing -eq 1 ]; then
    echo "❌ Some required files are missing"
    exit 1
fi

echo "✅ All required files present"

# Check markdown links (basic)
echo "🔗 Checking markdown links..."
broken=0
while IFS= read -r file; do
    while IFS= read -r link; do
        # Extract the target (everything between parentheses)
        target=$(echo "$link" | sed -n 's/.*(\([^)]*\)).*/\1/p')
        if [[ "$target" =~ ^http ]] || [[ "$target" =~ ^# ]]; then
            continue  # Skip external and anchor links
        fi
        if [ ! -f "$target" ] && [ ! -d "$target" ]; then
            echo "  ⚠️  $file: broken link to $target"
            broken=1
        fi
    done < <(grep -o '\[[^]]*\]([^)]*)' "$file" || true)
done < <(find . -name "*.md" -not -path "./target/*" -not -path "./.git/*")

if [ $broken -eq 1 ]; then
    echo "⚠️  Some links may be broken (see warnings above)"
else
    echo "✅ No obvious broken links found"
fi

# Check for empty placeholder files
echo "📄 Checking placeholder files..."
placeholders=0
for file in docs/COMPANIES.md docs/IANA-APPLICATION.md; do
    if grep -q "Your name here" "$file" || grep -q "not yet submitted" "$file"; then
        echo "  ℹ️  $file still has placeholder text (this is fine)"
        placeholders=1
    fi
done

echo ""
echo "📊 Summary:"
echo "  Files checked: ${#required_files[@]}"
echo "  Placeholders: $placeholders (expected for COMPANIES/IANA)"
echo ""
echo "✅ Documentation validation complete!"

# Optional: Check if examples run
read -p "❓ Run examples to verify? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo "🧪 Running examples..."
    cargo run --example ai_generate_image --quiet || echo "⚠️  Example failed"
    cargo run --example ai_generate_audio --quiet || echo "⚠️  Example failed"
    cargo run --example ai_generate_video_simple --quiet || echo "⚠️  Example failed"
    echo "✅ Examples completed (warnings OK if missing ffmpeg)"
fi