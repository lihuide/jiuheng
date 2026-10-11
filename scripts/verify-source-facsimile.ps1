[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [string]$SourcePath,

    [Parameter(Mandatory = $true)]
    [string]$ExpectedChecksum,

    [long]$ExpectedBytes = -1
)

$ErrorActionPreference = 'Stop'
if ($ExpectedChecksum -cnotmatch '^sha256:[0-9a-f]{64}$') {
    throw 'ExpectedChecksum must be a lowercase sha256: digest with exactly 64 hexadecimal digits.'
}
$resolvedSourcePath = [System.IO.Path]::GetFullPath($SourcePath)
if (-not (Test-Path -LiteralPath $resolvedSourcePath -PathType Leaf)) {
    throw "Facsimile file does not exist: $resolvedSourcePath"
}
$file = Get-Item -LiteralPath $resolvedSourcePath
if ($ExpectedBytes -ge 0 -and $file.Length -ne $ExpectedBytes) {
    throw "Facsimile byte length mismatch: expected $ExpectedBytes, got $($file.Length)."
}
$actualChecksum = 'sha256:' + (Get-FileHash -LiteralPath $resolvedSourcePath -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actualChecksum -cne $ExpectedChecksum) {
    throw "Facsimile SHA-256 mismatch: expected $ExpectedChecksum, got $actualChecksum."
}
Write-Output "Facsimile bytes verified: $($file.Length) bytes, $actualChecksum."
