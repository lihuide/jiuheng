[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$CargoArguments
)

$ErrorActionPreference = "Stop"
$VswherePath = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (-not (Test-Path -LiteralPath $VswherePath)) {
    throw "Visual Studio Installer (vswhere.exe) was not found."
}

$VisualStudioPath = & $VswherePath -latest -products * `
    -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
    -property installationPath
if ([string]::IsNullOrWhiteSpace($VisualStudioPath)) {
    throw "Visual Studio C++ Build Tools were not found."
}

$DeveloperCommand = Join-Path $VisualStudioPath "Common7\Tools\VsDevCmd.bat"
$DeveloperArguments = @(
    "/d",
    "/s",
    "/c",
    "`"$DeveloperCommand`" -no_logo -arch=x64 -host_arch=x64 && set"
)
$EnvironmentLines = & cmd.exe @DeveloperArguments
if ($LASTEXITCODE -ne 0) {
    throw "Failed to initialize the Visual Studio x64 developer environment."
}

foreach ($EnvironmentLine in $EnvironmentLines) {
    $Separator = $EnvironmentLine.IndexOf('=')
    if ($Separator -le 0) { continue }
    $VariableName = $EnvironmentLine.Substring(0, $Separator)
    $VariableValue = $EnvironmentLine.Substring($Separator + 1)
    [Environment]::SetEnvironmentVariable($VariableName, $VariableValue, 'Process')
}

$CargoExecutable = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
if (-not (Test-Path -LiteralPath $CargoExecutable)) {
    $CargoExecutable = (Get-Command cargo -ErrorAction Stop).Source
}

& $CargoExecutable @CargoArguments
exit $LASTEXITCODE
