[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version,
    [string]$TargetDirectory = "",
    [switch]$RefreshExisting
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
$SourceArchive = Join-Path $OutputsDir "Jiuheng-Source-$Version.zip"
$Checksums = Join-Path $OutputsDir "SHA256SUMS-$Version.txt"
$Staging = Join-Path $ProjectRoot "work\source-package-$Version"
$TemporarySourceArchive = Join-Path $ProjectRoot "work\Jiuheng-Source-$Version.new.zip"

$manifest = Get-Content -Raw (Join-Path $ProjectRoot "Cargo.toml")
if ($manifest -notmatch ('version\s*=\s*"' + [regex]::Escape($Version) + '"')) {
    throw "Cargo workspace version does not match requested release $Version."
}
foreach ($required in @($ReleaseExecutable, $Installer)) {
    if (-not (Test-Path -LiteralPath $required -PathType Leaf)) {
        throw "Required release artifact does not exist: $required"
    }
}
foreach ($output in @($Portable, $SourceArchive, $Checksums)) {
    if (-not $RefreshExisting -and (Test-Path -LiteralPath $output)) {
        throw "Refusing to overwrite an existing release artifact: $output"
    }
}
if (Test-Path -LiteralPath $Staging) {
    throw "Source staging directory already exists: $Staging"
}
if (Test-Path -LiteralPath $TemporarySourceArchive) {
    throw "Temporary source archive already exists: $TemporarySourceArchive"
}

New-Item -ItemType Directory -Path $OutputsDir -Force | Out-Null
Copy-Item -LiteralPath $ReleaseExecutable -Destination $Portable -Force

$sourceRoots = @(".cargo", "apps", "crates", "data", "docs", "packaging", "scripts")
$rootFiles = Get-ChildItem -LiteralPath $ProjectRoot -File
$sourceFiles = @($rootFiles)
foreach ($sourceRoot in $sourceRoots) {
    $sourceFiles += Get-ChildItem -LiteralPath (Join-Path $ProjectRoot $sourceRoot) -Recurse -File
}

try {
    New-Item -ItemType Directory -Path $Staging | Out-Null
    foreach ($file in $sourceFiles) {
        $relative = [System.IO.Path]::GetRelativePath($ProjectRoot, $file.FullName)
        $destination = Join-Path $Staging $relative
        $destinationDirectory = Split-Path -Parent $destination
        New-Item -ItemType Directory -Path $destinationDirectory -Force | Out-Null
        Copy-Item -LiteralPath $file.FullName -Destination $destination
    }
    Compress-Archive -Path (Join-Path $Staging "*") -DestinationPath $TemporarySourceArchive -CompressionLevel Optimal
    Move-Item -LiteralPath $TemporarySourceArchive -Destination $SourceArchive -Force
} finally {
    if (Test-Path -LiteralPath $Staging) {
        $resolvedStaging = [System.IO.Path]::GetFullPath($Staging)
        $expectedParent = [System.IO.Path]::GetFullPath((Join-Path $ProjectRoot "work"))
        if (-not $resolvedStaging.StartsWith($expectedParent + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Refusing to remove source staging outside the project work directory: $resolvedStaging"
        }
        [System.IO.Directory]::Delete($resolvedStaging, $true)
    }
    if (Test-Path -LiteralPath $TemporarySourceArchive) {
        [System.IO.File]::Delete($TemporarySourceArchive)
    }
}

$artifacts = @($Installer, $Portable, $SourceArchive)
$checksumLines = foreach ($artifact in $artifacts) {
    $hash = Get-FileHash -Algorithm SHA256 -LiteralPath $artifact
    "{0}  {1}" -f $hash.Hash, (Split-Path -Leaf $artifact)
}
[System.IO.File]::WriteAllLines($Checksums, $checksumLines, [System.Text.UTF8Encoding]::new($false))

foreach ($artifact in $artifacts + $Checksums) {
    $item = Get-Item -LiteralPath $artifact
    Write-Host "$($item.Name) ($($item.Length) bytes)"
}
