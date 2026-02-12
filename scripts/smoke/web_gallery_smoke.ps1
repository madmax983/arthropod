$ErrorActionPreference = "Stop"

$distDir = "target/trunk/widget_gallery_web"
$indexPath = Join-Path $distDir "index.html"

if (-not (Test-Path $distDir)) {
    throw "Missing trunk dist directory: $distDir"
}

if (-not (Test-Path $indexPath)) {
    throw "Missing built index.html at $indexPath"
}

$html = Get-Content $indexPath -Raw

if ($html -notmatch "<title>Arthropod Widget Gallery Live</title>") {
    throw "Expected page title 'Arthropod Widget Gallery Live' in built HTML."
}

if ($html -notmatch "id=""arthropod-canvas""") {
    throw "Expected #arthropod-canvas in built HTML."
}

$jsBundle = Get-ChildItem $distDir -Recurse -File -Filter "*.js" | Select-Object -First 1
if ($null -eq $jsBundle) {
    throw "No JavaScript bundle found in $distDir"
}

Write-Host "web_gallery_smoke: PASS"
