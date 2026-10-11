[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$InstallerPath,
    [Parameter(Mandatory = $true)]
    [string]$SmokeRoot,
    [int]$StartupSeconds = 5
)

$ErrorActionPreference = "Stop"
$InstallerPath = [System.IO.Path]::GetFullPath($InstallerPath)
$SmokeRoot = [System.IO.Path]::GetFullPath($SmokeRoot)
$AppDir = Join-Path $SmokeRoot "app"
$DataDir = Join-Path $SmokeRoot "data"
$InstallLog = Join-Path $SmokeRoot "install.log"

if (-not (Test-Path -LiteralPath $InstallerPath -PathType Leaf)) {
    throw "Installer does not exist: $InstallerPath"
}
if (Test-Path -LiteralPath $SmokeRoot) {
    $existing = Get-ChildItem -LiteralPath $SmokeRoot -Force | Select-Object -First 1
    if ($null -ne $existing) {
        throw "Smoke root must be absent or empty: $SmokeRoot"
    }
}
New-Item -ItemType Directory -Path $SmokeRoot -Force | Out-Null
New-Item -ItemType Directory -Path $DataDir -Force | Out-Null

$installerArguments = @(
    "/VERYSILENT",
    "/SUPPRESSMSGBOXES",
    "/NORESTART",
    "/SP-",
    "/DIR=$AppDir",
    "/LOG=$InstallLog"
)
$installerProcess = Start-Process -FilePath $InstallerPath -ArgumentList $installerArguments -Wait -PassThru -WindowStyle Hidden
if ($installerProcess.ExitCode -ne 0) {
    throw "Silent install failed with exit code $($installerProcess.ExitCode)."
}

$Executable = Join-Path $AppDir "jiuheng.exe"
if (-not (Test-Path -LiteralPath $Executable -PathType Leaf)) {
    throw "Installed executable is missing: $Executable"
}

$previousDataDir = $env:JIUHENG_DATA_DIR
try {
    $env:JIUHENG_DATA_DIR = $DataDir
    $appProcess = Start-Process -FilePath $Executable -PassThru -WindowStyle Hidden
    Start-Sleep -Seconds $StartupSeconds
    if ($appProcess.HasExited) {
        throw "Application exited during the startup survival window."
    }
    if (-not (Test-Path -LiteralPath (Join-Path $DataDir "jiuheng.sqlite3") -PathType Leaf)) {
        throw "Application did not create the isolated SQLite database."
    }
    Stop-Process -Id $appProcess.Id -Force
    $appProcess.WaitForExit()
} finally {
    $env:JIUHENG_DATA_DIR = $previousDataDir
}

$Uninstaller = Join-Path $AppDir "unins000.exe"
if (-not (Test-Path -LiteralPath $Uninstaller -PathType Leaf)) {
    throw "Uninstaller is missing: $Uninstaller"
}
$uninstallerProcess = Start-Process -FilePath $Uninstaller -ArgumentList @(
    "/VERYSILENT",
    "/SUPPRESSMSGBOXES",
    "/NORESTART"
) -Wait -PassThru -WindowStyle Hidden
if ($uninstallerProcess.ExitCode -ne 0) {
    throw "Silent uninstall failed with exit code $($uninstallerProcess.ExitCode)."
}
if (Test-Path -LiteralPath $Executable) {
    throw "Executable remains after uninstall: $Executable"
}

Write-Host "Smoke test passed: install, startup, isolated SQLite creation, uninstall."
