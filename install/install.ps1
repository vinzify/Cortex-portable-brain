param(
  [string]$Version = "latest",
  [string]$Repo = "vinzify/Cortex-portable-brain",
  [string]$InstallDir = "$HOME\\AppData\\Local\\Programs\\cortex"
)

$ErrorActionPreference = "Stop"
$os = "windows"
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }
$asset = "cortex-app-$os-$arch.exe"

if ($Version -eq "latest") {
  $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
  $tag = $release.tag_name
} else {
  $tag = $Version
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$base = "https://github.com/$Repo/releases/download/$tag"
Invoke-WebRequest -Uri "$base/$asset" -OutFile "$InstallDir\\cortex.exe"
Invoke-WebRequest -Uri "$base/$asset.sha256" -OutFile "$env:TEMP\\cortex.sha256"

$expected = (Get-Content "$env:TEMP\\cortex.sha256" -Raw).Split(' ')[0].Trim().ToLowerInvariant()
$actual = (Get-FileHash "$InstallDir\\cortex.exe" -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -ne $actual) {
  throw "Checksum mismatch for cortex.exe expected=$expected actual=$actual"
}

Write-Host "Installed cortex to $InstallDir\\cortex.exe"
Write-Host "Add $InstallDir to PATH if needed."
