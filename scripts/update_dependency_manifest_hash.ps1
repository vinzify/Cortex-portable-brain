$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$outPath = Join-Path $repoRoot ".ci\dependency-manifest.sha256"

$files = @("Cargo.toml", "Cargo.lock")
Get-ChildItem -Recurse -Filter Cargo.toml (Join-Path $repoRoot "crates") | ForEach-Object {
  $rel = $_.FullName.Substring($repoRoot.Length + 1).Replace("\", "/")
  $files += $rel
}
$files = $files | Sort-Object

$builder = New-Object System.Text.StringBuilder
foreach ($rel in $files) {
  [void]$builder.Append("--- ").Append($rel).Append(" ---`n")
  $raw = Get-Content (Join-Path $repoRoot $rel) -Raw
  $normalized = $raw.Replace("`r`n", "`n")
  [void]$builder.Append($normalized).Append("`n")
}

$utf8 = New-Object System.Text.UTF8Encoding($false)
$bytes = $utf8.GetBytes($builder.ToString())
$sha = [System.Security.Cryptography.SHA256]::Create()
$actualBytes = $sha.ComputeHash($bytes)
$actual = ([System.BitConverter]::ToString($actualBytes)).Replace("-", "").ToUpperInvariant()

New-Item -ItemType Directory -Force (Join-Path $repoRoot ".ci") | Out-Null
[System.IO.File]::WriteAllText($outPath, "$actual`n", ([System.Text.UTF8Encoding]::new($false)))

Write-Output "Wrote $outPath = $actual"
