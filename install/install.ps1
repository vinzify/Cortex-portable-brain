param(
  [string]$Version = "latest",
  [string]$Repo = "vinzify/Cortex-portable-brain",
  [string]$InstallDir = "$HOME\\AppData\\Local\\Programs\\cortex"
)

$ErrorActionPreference = "Stop"
$os = "windows"
$arch = if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { "arm64" } else { "x64" }
$cortexAsset = "cortex-app-$os-$arch.exe"
$rmvmAsset = "rmvm-grpc-server-$os-$arch.exe"

if ($Version -eq "latest") {
  $api = "https://api.github.com/repos/$Repo/releases"
  $latestUrl = "{0}/latest" -f $api
  $listUrl = "{0}?per_page=20" -f $api
  try {
    $release = Invoke-RestMethod -Uri $latestUrl
  } catch {
    # `latest` endpoint excludes pre-releases; fall back to newest non-draft release.
    $releases = Invoke-RestMethod -Uri $listUrl
    $release = $releases | Where-Object { -not $_.draft } | Select-Object -First 1
    if ($null -eq $release) {
      throw "No published releases found for $Repo."
    }
  }
  $tag = $release.tag_name
} else {
  $tag = $Version
}

New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
$base = "https://github.com/$Repo/releases/download/$tag"
Invoke-WebRequest -Uri "$base/$cortexAsset" -OutFile "$InstallDir\\cortex.exe"
Invoke-WebRequest -Uri "$base/$cortexAsset.sha256" -OutFile "$env:TEMP\\cortex.sha256"
Invoke-WebRequest -Uri "$base/$rmvmAsset" -OutFile "$InstallDir\\rmvm-grpc-server.exe"
Invoke-WebRequest -Uri "$base/$rmvmAsset.sha256" -OutFile "$env:TEMP\\rmvm.sha256"

$expected = (Get-Content "$env:TEMP\\cortex.sha256" -Raw).Split(' ')[0].Trim().ToLowerInvariant()
$actual = (Get-FileHash "$InstallDir\\cortex.exe" -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expected -ne $actual) {
  throw "Checksum mismatch for cortex.exe expected=$expected actual=$actual"
}
$expectedRmvm = (Get-Content "$env:TEMP\\rmvm.sha256" -Raw).Split(' ')[0].Trim().ToLowerInvariant()
$actualRmvm = (Get-FileHash "$InstallDir\\rmvm-grpc-server.exe" -Algorithm SHA256).Hash.ToLowerInvariant()
if ($expectedRmvm -ne $actualRmvm) {
  throw "Checksum mismatch for rmvm-grpc-server.exe expected=$expectedRmvm actual=$actualRmvm"
}

Write-Host "Installed cortex to $InstallDir\\cortex.exe"
Write-Host "Installed rmvm-grpc-server to $InstallDir\\rmvm-grpc-server.exe"
# Ensure user PATH contains install dir so `cortex` works in new terminals.
try {
  $userPathRaw = [Environment]::GetEnvironmentVariable("Path", "User")
  $userPathEntries = @()
  if (-not [string]::IsNullOrWhiteSpace($userPathRaw)) {
    $userPathEntries = $userPathRaw.Split(';') | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
  }
  $normalizedInstallDir = $InstallDir.Trim().TrimEnd('\')
  $presentInUserPath = $false
  foreach ($entry in $userPathEntries) {
    if ($entry.Trim().TrimEnd('\').Equals($normalizedInstallDir, [System.StringComparison]::OrdinalIgnoreCase)) {
      $presentInUserPath = $true
      break
    }
  }
  if (-not $presentInUserPath) {
    $newUserPath = (($userPathEntries + $InstallDir) -join ';')
    [Environment]::SetEnvironmentVariable("Path", $newUserPath, "User")
    Write-Host "Added $InstallDir to user PATH."
  }
  if (-not ($env:Path.Split(';') | Where-Object { $_.Trim().TrimEnd('\').Equals($normalizedInstallDir, [System.StringComparison]::OrdinalIgnoreCase) })) {
    $env:Path = "$env:Path;$InstallDir"
  }
} catch {
  Write-Warning "Could not update PATH automatically. Add $InstallDir to PATH manually."
}
Write-Host "If cortex is still not found, open a new terminal window."
if ($Host.Name -ne "ServerRemoteHost") {
  Write-Host "Running guided setup..."
  & "$InstallDir\\cortex.exe" setup
}
