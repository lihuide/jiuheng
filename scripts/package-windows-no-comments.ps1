[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version,
    [Parameter(Mandatory = $true)]
    [string]$PythonPath,
    [string]$TargetDirectory = ""
)

$ErrorActionPreference = "Stop"
$ProjectRoot = [System.IO.Path]::GetFullPath((Split-Path -Parent $PSScriptRoot))
$OutputsDir = Join-Path $ProjectRoot "outputs"
if ([string]::IsNullOrWhiteSpace($TargetDirectory)) {
    $TargetDirectory = Join-Path $ProjectRoot "target"
}
$TargetDirectory = [System.IO.Path]::GetFullPath($TargetDirectory)
$ReleaseExecutable = Join-Path $TargetDirectory "release\jiuheng.exe"
$Installer = Join-Path $OutputsDir "Jiuheng-Setup-x64-$Version.exe"
$Portable = Join-Path $OutputsDir "Jiuheng-Portable-x64-$Version.exe"
$SourceArchive = Join-Path $OutputsDir "Jiuheng-Source-NoComments-$Version.zip"
$Checksums = Join-Path $OutputsDir "SHA256SUMS-$Version.txt"
$Packager = Join-Path $PSScriptRoot "package-source.py"

$manifest = Get-Content -Raw (Join-Path $ProjectRoot "Cargo.toml")
if ($manifest -notmatch ('version\s*=\s*"' + [regex]::Escape($Version) + '"')) {
    throw "Cargo workspace version does not match requested release $Version."
}
foreach ($required in @($PythonPath, $ReleaseExecutable, $Installer, $Packager)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw "Required release input does not exist: $required"
    }
}
foreach ($output in @($Portable, $SourceArchive, $Checksums)) {
    if (Test-Path -LiteralPath $output) {
        throw "Refusing to overwrite an existing release artifact: $output"
    }
}

New-Item -ItemType Directory -Path $OutputsDir -Force | Out-Null
Copy-Item -LiteralPath $ReleaseExecutable -Destination $Portable
& $PythonPath $Packager pack --root $ProjectRoot --output $SourceArchive --without-comments
if ($LASTEXITCODE -ne 0) {
    throw "Comment-free source package failed verification."
}

$artifacts = @($Installer, $Portable, $SourceArchive)
$checksumLines = foreach ($artifact in $artifacts) {
    $hash = Get-FileHash -Algorithm SHA256 -LiteralPath $artifact
    "{0}  {1}" -f $hash.Hash.ToLowerInvariant(), (Split-Path -Leaf $artifact)
}
[System.IO.File]::WriteAllLines($Checksums, $checksumLines, [System.Text.UTF8Encoding]::new($false))

foreach ($artifact in $artifacts + $Checksums) {
    $item = Get-Item -LiteralPath $artifact
    Write-Host "$($item.Name) ($($item.Length) bytes)"
}
