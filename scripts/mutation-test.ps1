# Run mutation testing locally (Windows PowerShell)
# Usage: .\scripts\mutation-test.ps1 [-Package package-name]

param(
    [string]$Package = ""
)

Write-Host "🧬 Running Mutation Testing..." -ForegroundColor Cyan
Write-Host ""

# Install cargo-mutants if not present
if (-not (Get-Command cargo-mutants -ErrorAction SilentlyContinue)) {
    Write-Host "Installing cargo-mutants..." -ForegroundColor Yellow
    cargo install cargo-mutants
}

# Run mutation testing
if ($Package -eq "") {
    Write-Host "Running on all packages..." -ForegroundColor Green
    cargo mutants --no-shuffle --timeout 300 -- --all-features
} else {
    Write-Host "Running on package: $Package..." -ForegroundColor Green
    cargo mutants -p $Package --no-shuffle --timeout 300 -- --all-features
}

Write-Host ""
Write-Host "✅ Mutation testing complete!" -ForegroundColor Green
Write-Host "Results saved to: mutants.out/"
Write-Host ""
Write-Host "View results:"
Write-Host "  - Markdown report: mutants.out/mutants.md"
Write-Host "  - JSON report: mutants.out.json"
Write-Host ""

# Check mutation score
if (Test-Path "mutants.out.json") {
    $results = Get-Content "mutants.out.json" | ConvertFrom-Json
    $score = ($results.caught / $results.total_mutants) * 100
    Write-Host ("Mutation Score: {0:N1}%" -f $score) -ForegroundColor Cyan

    if ($score -lt 80.0) {
        Write-Host "⚠️  Score below 80% threshold" -ForegroundColor Yellow
    } else {
        Write-Host "🎉 Score above 80% threshold" -ForegroundColor Green
    }
}
