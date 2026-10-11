[CmdletBinding()]
param(
    [string]$Configuration = "release",
    [string]$IsccPath = "",
    [string]$TargetDirectory = ""
)

$ErrorActionPreference = "Stop"
$ProjectRoot = Split-Path -Parent $PSScriptRoot
$OutputsDir = Join-Path $ProjectRoot "outputs"
$StagingDir = Join-Path $ProjectRoot "work\package"
$StagedExecutablePath = Join-Path $StagingDir "jiuheng.exe"
$InstallerScript = Join-Path $ProjectRoot "packaging\jiuheng.iss"

if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
    $TargetDirectory = Join-Path $ProjectRoot "target"
}
$TargetDirectory = [System.IO.Path]::GetFullPath($TargetDirectory)
$env:CARGO_TARGET_DIR = $TargetDirectory
$ExecutablePath = Join-Path $TargetDirectory "release\jiuheng.exe"

Push-Location $ProjectRoot
try {
    & "$PSScriptRoot\Invoke-CargoMsvc.ps1" test --workspace
    if ($LASTEXITCODE -ne 0) { throw "Workspace tests failed." }

    & "$PSScriptRoot\Invoke-CargoMsvc.ps1" build --workspace --release
    if ($LASTEXITCODE -ne 0) { throw "Release build failed." }
} finally {
    Pop-Location
}

if (-not (Test-Path -LiteralPath $ExecutablePath)) {
    throw "Release executable was not produced: $ExecutablePath"
}

New-Item -ItemType Directory -Path $StagingDir -Force | Out-Null
Copy-Item -LiteralPath $ExecutablePath -Destination $StagedExecutablePath -Force

$ExecutableSize = (Get-Item -LiteralPath $ExecutablePath).Length
if ($ExecutableSize -gt 100MB) {
    throw "Release executable exceeds the 100 MB product target."
}

if ([string]::IsNullOrWhiteSpace($IsccPath)) {
    $Candidates = @(
        "$env:LOCALAPPDATA\Programs\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )
    $IsccPath = $Candidates | Where-Object { Test-Path -LiteralPath $_ } | Select-Object -First 1
}

if ([string]::IsNullOrWhiteSpace($IsccPath) -or -not (Test-Path -LiteralPath $IsccPath)) {
    throw "Inno Setup 6 compiler was not found. Pass -IsccPath with the full ISCC.exe path."
}

New-Item -ItemType Directory -Path $OutputsDir -Force | Out-Null
& $IsccPath $InstallerScript
if ($LASTEXITCODE -ne 0) { throw "Installer build failed." }

$Installer = Get-ChildItem -LiteralPath $OutputsDir -Filter "Jiuheng-Setup-x64-*.exe" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1
if ($null -eq $Installer) { throw "Installer output was not found." }
if ($Installer.Length -gt 100MB) { throw "Installer exceeds the 100 MB product target." }

Write-Host "Executable: $ExecutablePath ($([math]::Round($ExecutableSize / 1MB, 2)) MB)"
Write-Host "Installer:  $($Installer.FullName) ($([math]::Round($Installer.Length / 1MB, 2)) MB)"
