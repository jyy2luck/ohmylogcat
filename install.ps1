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
# Direct latest/download URL (avoids unauthenticated GitHub REST API rate limits).
$LatestDownloadUrl = "https://github.com/$Repo/releases/latest/download/$Asset"
$RetrySeconds = 60
$retryOverride = 0
if ($env:OHMYLOGCAT_UPDATE_RETRY_SECONDS -and
    [int]::TryParse($env:OHMYLOGCAT_UPDATE_RETRY_SECONDS, [ref]$retryOverride) -and
    $retryOverride -gt 0) {
    $RetrySeconds = $retryOverride
}

function New-UniqueTempPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Prefix,
        [Parameter(Mandatory = $true)]
        [string]$Extension
    )

    Join-Path ([System.IO.Path]::GetTempPath()) (
        $Prefix + "-" + [guid]::NewGuid().ToString("N") + $Extension
    )
}

function Quote-ProcessArgument {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Value
    )

    '"' + $Value.Replace('"', '\"') + '"'
}

function Write-InstallResult {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet("installed", "scheduled")]
        [string]$Result
    )

    Write-Host "ohmylogcat-install-result: $Result"
}

function Test-ValidExecutable {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $false
    }

    $item = Get-Item -LiteralPath $Path
    if ($item.Length -lt 2) {
        return $false
    }

    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $first = $stream.ReadByte()
        $second = $stream.ReadByte()
    } finally {
        $stream.Dispose()
    }
    return $first -eq 0x4d -and $second -eq 0x5a
}

function Try-RecoverableReplacement {
    param(
        [Parameter(Mandatory = $true)]
        [string]$StagedPath,
        [Parameter(Mandatory = $true)]
        [string]$TargetPath,
        [Parameter(Mandatory = $true)]
        [string]$BackupPath
    )

    $hadTarget = Test-Path -LiteralPath $TargetPath -PathType Leaf
    if ($hadTarget) {
        try {
            Move-Item -LiteralPath $TargetPath -Destination $BackupPath -Force -ErrorAction Stop
        } catch {
            # A running Windows executable normally fails here because its
            # image cannot be renamed or deleted while the process is alive.
            return $false
        }
    }

    try {
        Move-Item -LiteralPath $StagedPath -Destination $TargetPath -Force -ErrorAction Stop
    } catch {
        if ($hadTarget -and (Test-Path -LiteralPath $BackupPath -PathType Leaf)) {
            Move-Item -LiteralPath $BackupPath -Destination $TargetPath -Force -ErrorAction Stop
        }
        throw
    }

    if ($hadTarget -and (Test-Path -LiteralPath $BackupPath -PathType Leaf)) {
        Remove-Item -LiteralPath $BackupPath -Force -ErrorAction SilentlyContinue
    }
    return $true
}

function Get-DeferredHelperScript {
    @'
param(
    [Parameter(Mandatory = $true)]
    [string]$PendingPath,
    [Parameter(Mandatory = $true)]
    [string]$TargetPath,
    [Parameter(Mandatory = $true)]
    [string]$BackupPath,
    [Parameter(Mandatory = $true)]
    [string]$StatusPath,
    [Parameter(Mandatory = $true)]
    [string]$DeadlineUtc
)

$ErrorActionPreference = "Stop"
$deadline = [DateTime]::Parse($DeadlineUtc).ToUniversalTime()
$lastError = "target remained unavailable"

function Test-ValidExecutable {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (-not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        return $false
    }

    $item = Get-Item -LiteralPath $Path
    if ($item.Length -lt 2) {
        return $false
    }

    $stream = [System.IO.File]::OpenRead($Path)
    try {
        $first = $stream.ReadByte()
        $second = $stream.ReadByte()
    } finally {
        $stream.Dispose()
    }
    return $first -eq 0x4d -and $second -eq 0x5a
}

function Write-HelperStatus {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Result,
        [Parameter(Mandatory = $true)]
        [string]$Message
    )

    $lines = @(
        "result=$Result"
        "message=$Message"
        "target=$TargetPath"
        "pending=$PendingPath"
        "backup=$BackupPath"
        "timestamp=$(([DateTime]::UtcNow).ToString('o'))"
    )
    $lines | Set-Content -LiteralPath $StatusPath -Encoding UTF8
    Write-Host "ohmylogcat-install-result: $Result"
    Write-Host $Message
}

function Restore-Backup {
    if ((Test-Path -LiteralPath $BackupPath -PathType Leaf) -and
        -not (Test-Path -LiteralPath $TargetPath -PathType Leaf)) {
        Move-Item -LiteralPath $BackupPath -Destination $TargetPath -Force -ErrorAction Stop
    }
}

function Try-RecoverableReplacement {
    $hadTarget = Test-Path -LiteralPath $TargetPath -PathType Leaf
    if ($hadTarget) {
        try {
            Move-Item -LiteralPath $TargetPath -Destination $BackupPath -Force -ErrorAction Stop
        } catch {
            return $false
        }
    }

    try {
        Move-Item -LiteralPath $PendingPath -Destination $TargetPath -Force -ErrorAction Stop
    } catch {
        if ($hadTarget -and (Test-Path -LiteralPath $BackupPath -PathType Leaf)) {
            Restore-Backup
        }
        throw
    }

    if ($hadTarget -and (Test-Path -LiteralPath $BackupPath -PathType Leaf)) {
        Remove-Item -LiteralPath $BackupPath -Force -ErrorAction SilentlyContinue
    }
    return $true
}

if (-not (Test-ValidExecutable -Path $PendingPath)) {
    $lastError = "staged executable is missing or invalid"
} else {
    $installed = $false
    while ([DateTime]::UtcNow -lt $deadline) {
        try {
            if (Try-RecoverableReplacement) {
                $installed = $true
                break
            }
            $lastError = "target is still locked"
        } catch {
            $lastError = $_.Exception.Message
        }

        Start-Sleep -Milliseconds 250
    }

    if ($installed) {
        $message = "Deferred update completed: $TargetPath"
        Write-HelperStatus -Result "installed" -Message $message
        Remove-Item -LiteralPath $PendingPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $BackupPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $StatusPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
        exit 0
    }
}

try {
    Restore-Backup
} catch {
    $lastError = "$lastError; backup restoration failed: $($_.Exception.Message)"
}

$message = "Deferred update failed for $TargetPath before the retry deadline. $lastError Pending staged executable: $PendingPath. Status log: $StatusPath"
Write-HelperStatus -Result "failed" -Message $message
Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
exit 1
'@
}

$sourceOverride = $env:OHMYLOGCAT_INSTALL_SOURCE
$url = if ($sourceOverride) { $null } else { $LatestDownloadUrl }

$tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("ohmylogcat-install-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tmpdir | Out-Null
$pendingPath = New-UniqueTempPath -Prefix "ohmylogcat-pending" -Extension ".exe"
$backupPath = New-UniqueTempPath -Prefix "ohmylogcat-backup" -Extension ".exe"
$helperPath = New-UniqueTempPath -Prefix "ohmylogcat-helper" -Extension ".ps1"
$statusPath = New-UniqueTempPath -Prefix "ohmylogcat-update" -Extension ".log"
$scheduled = $false

try {
    $zipPath = Join-Path $tmpdir $Asset
    if ($sourceOverride) {
        Write-Host "Using local release archive $sourceOverride..."
        if (-not (Test-Path -LiteralPath $sourceOverride -PathType Leaf)) {
            throw "Local install source not found: $sourceOverride"
        }
        Copy-Item -LiteralPath $sourceOverride -Destination $zipPath -Force
    } else {
        Write-Host "Downloading $Asset..."
        Write-Host "  from $url"
        try {
            Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing
        } catch {
            throw "Download failed for $url. $($_.Exception.Message) Check network access and that a Release publishes asset: $Asset"
        }
    }

    Write-Host "Extracting..."
    Expand-Archive -Path $zipPath -DestinationPath $tmpdir -Force

    $exe = Get-ChildItem -Path $tmpdir -Filter "ohmylogcat.exe" -Recurse | Select-Object -First 1
    if (-not $exe) {
        throw "Archive did not contain ohmylogcat.exe"
    }

    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
    $dest = Join-Path $InstallDir "ohmylogcat.exe"
    Copy-Item -LiteralPath $exe.FullName -Destination $pendingPath -Force
    if (-not (Test-ValidExecutable -Path $pendingPath)) {
        throw "Downloaded archive did not contain a valid Windows executable"
    }

    $replaced = Try-RecoverableReplacement `
        -StagedPath $pendingPath `
        -TargetPath $dest `
        -BackupPath $backupPath

    if ($replaced) {
        Write-Host "Installed to $dest"
        Write-InstallResult -Result "installed"
    } else {
        $deadline = [DateTime]::UtcNow.AddSeconds($RetrySeconds)
        $deadlineText = $deadline.ToString("o", [Globalization.CultureInfo]::InvariantCulture)
        $helperContent = Get-DeferredHelperScript
        Set-Content -LiteralPath $helperPath -Value $helperContent -Encoding UTF8

        $initialStatus = @(
            "result=scheduled"
            "message=Waiting for the target executable to become available"
            "target=$dest"
            "pending=$pendingPath"
            "backup=$backupPath"
            "timestamp=$(([DateTime]::UtcNow).ToString('o'))"
        )
        $initialStatus | Set-Content -LiteralPath $statusPath -Encoding UTF8

        $powerShellName = if ($PSVersionTable.PSEdition -eq "Core") {
            "pwsh.exe"
        } else {
            "powershell.exe"
        }
        $powerShellPath = Join-Path $PSHOME $powerShellName
        $arguments = @(
            "-NoProfile"
            "-ExecutionPolicy"
            "Bypass"
            "-File"
            (Quote-ProcessArgument $helperPath)
            "-PendingPath"
            (Quote-ProcessArgument $pendingPath)
            "-TargetPath"
            (Quote-ProcessArgument $dest)
            "-BackupPath"
            (Quote-ProcessArgument $backupPath)
            "-StatusPath"
            (Quote-ProcessArgument $statusPath)
            "-DeadlineUtc"
            (Quote-ProcessArgument $deadlineText)
        )

        try {
            Start-Process `
                -FilePath $powerShellPath `
                -ArgumentList $arguments `
                -WindowStyle Hidden `
                -ErrorAction Stop | Out-Null
        } catch {
            throw "Could not start deferred updater: $($_.Exception.Message)"
        }

        $scheduled = $true
        Write-Host "Update scheduled; the target is locked and will be replaced after it becomes available."
        Write-Host "Deferred status log: $statusPath"
        Write-InstallResult -Result "scheduled"
    }

    if (-not $env:OHMYLOGCAT_INSTALL_SKIP_PATH) {
        $pathEntries = [Environment]::GetEnvironmentVariable("Path", "User") -split ";" | Where-Object { $_ }
        if ($pathEntries -notcontains $InstallDir) {
            $newPath = ($pathEntries + $InstallDir) -join ";"
            [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
            $env:Path = "$InstallDir;$env:Path"
            Write-Host "Added $InstallDir to your user PATH (new terminals will pick it up)."
        }
    }

    Write-Host "Requires adb (Android SDK platform-tools) on PATH or configured in Settings."
    Write-Host "Run: ohmylogcat"
} catch {
    if (-not $scheduled) {
        Remove-Item -LiteralPath $pendingPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $backupPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $helperPath -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath $statusPath -Force -ErrorAction SilentlyContinue
    }
    throw
}
finally {
    Remove-Item -Recurse -Force $tmpdir -ErrorAction SilentlyContinue
}
