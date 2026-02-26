$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$lockPath = Join-Path $repoRoot "core_version.lock"
if (!(Test-Path $lockPath)) {
  throw "Missing lock file: $lockPath"
}

$lock = Get-Content $lockPath -Raw | ConvertFrom-Json
if ($lock.proto_version -ne "cortex_rmvm_v3_1") {
  throw "Unexpected proto_version in core_version.lock: $($lock.proto_version)"
}

function Get-CanonicalTextSha256([string]$Text) {
  $normalized = $Text.Replace("`r`n", "`n").Replace("`r", "`n")
  $bytes = ([System.Text.UTF8Encoding]::new($false)).GetBytes($normalized)
  $sha = [System.Security.Cryptography.SHA256]::Create()
  try {
    $hashBytes = $sha.ComputeHash($bytes)
  } finally {
    $sha.Dispose()
  }
  return ([System.BitConverter]::ToString($hashBytes)).Replace("-", "").ToLowerInvariant()
}

$checksums = $lock.proto_checksums.PSObject.Properties
if (!$checksums -or $checksums.Count -eq 0) {
  throw "Missing proto_checksums in core_version.lock"
}

foreach ($entry in $checksums) {
  $path = $entry.Name
  $expected = $entry.Value.ToString().ToLowerInvariant()
  $fileName = [System.IO.Path]::GetFileName($path)
  $localPath = Join-Path $repoRoot "third_party\core-proto\$fileName"
  if (!(Test-Path $localPath)) {
    throw "Missing local proto mirror: $localPath"
  }
  $content = Get-Content $localPath -Raw
  $actual = Get-CanonicalTextSha256 $content
  if ($actual -ne $expected) {
    throw "Proto checksum mismatch for $path expected=$expected actual=$actual"
  }
}

Write-Output "Core proto checksums match core_version.lock (local mirror verified)."
