$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$lockPath = Join-Path $repoRoot "core_version.lock"
$cargoPath = Join-Path $repoRoot "Cargo.toml"

if (!(Test-Path $lockPath)) {
  throw "Missing lock file: $lockPath"
}
if (!(Test-Path $cargoPath)) {
  throw "Missing Cargo.toml at $cargoPath"
}

$lock = Get-Content $lockPath -Raw | ConvertFrom-Json
foreach ($key in @("core_repo", "core_tag", "core_commit", "core_dependency_manifest_sha256")) {
  if ([string]::IsNullOrWhiteSpace($lock.$key)) {
    throw "Missing required key '$key' in core_version.lock"
  }
}

$cargo = Get-Content $cargoPath -Raw
$depRegexes = @(
  @{ name = "rmvm-grpc"; pattern = 'rmvm-grpc\s*=\s*\{[^}]*git\s*=\s*"([^"]+)"[^}]*rev\s*=\s*"([^"]+)"' },
  @{ name = "rmvm-proto"; pattern = 'rmvm-proto\s*=\s*\{[^}]*git\s*=\s*"([^"]+)"[^}]*rev\s*=\s*"([^"]+)"' }
)

foreach ($dep in $depRegexes) {
  $m = [regex]::Match($cargo, $dep.pattern)
  if (!$m.Success) {
    throw "Dependency '$($dep.name)' must use explicit git + rev in Cargo.toml"
  }
  $gitUrl = $m.Groups[1].Value
  $rev = $m.Groups[2].Value
  if ($rev -ne $lock.core_commit) {
    throw "Dependency '$($dep.name)' rev mismatch. expected=$($lock.core_commit) actual=$rev"
  }
  if ($gitUrl -notmatch [regex]::Escape($lock.core_repo)) {
    throw "Dependency '$($dep.name)' git URL does not include core repo '$($lock.core_repo)'"
  }
}

$mirrorPath = Join-Path $repoRoot "third_party\core\dependency-manifest.sha256"
if (!(Test-Path $mirrorPath)) {
  throw "Missing local core dependency manifest mirror: $mirrorPath"
}
$actual = (Get-Content $mirrorPath -Raw).Trim().ToUpperInvariant()
$expected = $lock.core_dependency_manifest_sha256.Trim().ToUpperInvariant()
if ($actual -ne $expected) {
  throw "core_dependency_manifest_sha256 mismatch. expected=$expected actual=$actual"
}

Write-Output "Core version lock is valid for commit $($lock.core_commit) and tag $($lock.core_tag) (local mirror verified)."
