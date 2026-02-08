#!/bin/bash
# Run mutation testing locally
# Usage: ./scripts/mutation-test.sh [package]

set -e

PACKAGE="${1:-}"

echo "🧬 Running Mutation Testing..."
echo ""

# Install cargo-mutants if not present
if ! command -v cargo-mutants &> /dev/null; then
    echo "Installing cargo-mutants..."
    cargo install cargo-mutants
fi

# Run mutation testing
if [ -z "$PACKAGE" ]; then
    echo "Running on all packages..."
    cargo mutants --no-shuffle --timeout 300 -- --all-features
else
    echo "Running on package: $PACKAGE..."
    cargo mutants -p "$PACKAGE" --no-shuffle --timeout 300 -- --all-features
fi

echo ""
echo "✅ Mutation testing complete!"
echo "Results saved to: mutants.out/"
echo ""
echo "View results:"
echo "  - HTML report: mutants.out/index.html"
echo "  - JSON report: mutants.out.json"
echo ""

# Check mutation score
if [ -f "mutants.out.json" ]; then
    SCORE=$(jq -r '(.caught / .total_mutants * 100)' mutants.out.json)
    echo "Mutation Score: $SCORE%"

    if (( $(echo "$SCORE < 80.0" | bc -l) )); then
        echo "⚠️  Score below 80% threshold"
    else
        echo "🎉 Score above 80% threshold"
    fi
fi
