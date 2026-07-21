# Install ohmylogcat from the latest GitHub Release.
# Usage (PowerShell):
#   irm https://raw.githubusercontent.com/jyy2luck/ohmylogcat/main/install.ps1 | iex
# Optional:
#   $env:INSTALL_DIR = "$env:LOCALAPPDATA\ohmylogcat"
#   irm ... | iex

$ErrorActionPreference = "Stop"

$Repo = "jyy2luck/ohmylogcat"
$Asset = "ohmylogcat-x86_64-pc-windows-msvc.zip"
$InstallDir = if ($env:INSTALL_DIR) { $env:INSTALL_DIR } else { Join-Path $env:LOCALAPPDATA "ohmylogcat" }
$Api = "https://api.github.com/repos/$Repo/releases/latest"

Write-Host "Fetching latest release metadata..."
$headers = @{
    "User-Agent" = "ohmylogcat-install"
    "Accept"     = "application/vnd.github+json"
}
$release = Invoke-RestMethod -Uri $Api -Headers $headers
$assetInfo = $release.assets | Where-Object { $_.name -eq $Asset } | Select-Object -First 1

if (-not $assetInfo) {
    $tag = $release.tag_name
    if (-not $tag) {
        throw "Could not find a GitHub release. Publish a tag like v0.1.0 first."
    }
    $url = "https://github.com/$Repo/releases/download/$tag/$Asset"
} else {
    $url = $assetInfo.browser_download_url
}

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("ohmylogcat-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmpdir | Out-Null

try {
    $zipPath = Join-Path $tmpdir $Asset
    Write-Host "Downloading $Asset..."
    Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing

    Write-Host "Extracting..."
    Expand-Archive -Path $zipPath -DestinationPath $tmpdir -Force

    $exe = Get-ChildItem -Path $tmpdir -Filter "ohmylogcat.exe" -Recurse | Select-Object -First 1
    if (-not $exe) {
        throw "Archive did not contain ohmylogcat.exe"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $dest = Join-Path $InstallDir "ohmylogcat.exe"
    Copy-Item -Path $exe.FullName -Destination $dest -Force

    Write-Host "Installed to $dest"

    $pathEntries = [Environment]::GetEnvironmentVariable("Path", "User") -split ";" | Where-Object { $_ }
    if ($pathEntries -notcontains $InstallDir) {
        $newPath = ($pathEntries + $InstallDir) -join ";"
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        $env:Path = "$InstallDir;$env:Path"
        Write-Host "Added $InstallDir to your user PATH (new terminals will pick it up)."
    }

    Write-Host "Requires adb (Android SDK platform-tools) on PATH or configured in Settings."
    Write-Host "Run: ohmylogcat"
}
finally {
    Remove-Item -Recurse -Force $tmpdir -ErrorAction SilentlyContinue
}
