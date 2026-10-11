param(
    [Parameter(Mandatory = $true)]
    [string]$SourceRoot,

    [string]$ManifestPath = (Join-Path $PSScriptRoot "..\data\research\reference-source-manifest.json"),

    [string]$GitPath = "git"
)

$ErrorActionPreference = "Stop"

function Assert-Equal {
    param(
        [Parameter(Mandatory = $true)]$Actual,
        [Parameter(Mandatory = $true)]$Expected,
        [Parameter(Mandatory = $true)][string]$Label
    )

    if ($Actual -ne $Expected) {
        throw "$Label mismatch: expected '$Expected', got '$Actual'"
    }
}

$resolvedSourceRoot = [System.IO.Path]::GetFullPath($SourceRoot)
$resolvedManifestPath = [System.IO.Path]::GetFullPath($ManifestPath)
if (-not (Test-Path -LiteralPath $resolvedSourceRoot -PathType Container)) {
    throw "Reference source root does not exist: $resolvedSourceRoot"
}
if (-not (Test-Path -LiteralPath $resolvedManifestPath -PathType Leaf)) {
    throw "Reference source manifest does not exist: $resolvedManifestPath"
}

$manifest = Get-Content -LiteralPath $resolvedManifestPath -Raw -Encoding UTF8 | ConvertFrom-Json
Assert-Equal $manifest.schema_version 1 "manifest schema version"

$verifiedProjects = 0
$verifiedFiles = 0
foreach ($project in $manifest.projects) {
    $repositoryPath = [System.IO.Path]::GetFullPath((Join-Path $resolvedSourceRoot $project.project_id))
    $expectedRepositoryPath = $resolvedSourceRoot + [System.IO.Path]::DirectorySeparatorChar + $project.project_id
    Assert-Equal $repositoryPath $expectedRepositoryPath "repository path"
    if (-not (Test-Path -LiteralPath $repositoryPath -PathType Container)) {
        throw "Pinned repository is missing: $repositoryPath"
    }

    $actualCommit = (& $GitPath -C $repositoryPath rev-parse HEAD).Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to read commit for $($project.project_id)"
    }
    Assert-Equal $actualCommit $project.commit_sha "$($project.project_id) commit"

    $actualTree = (& $GitPath -C $repositoryPath rev-parse "HEAD^{tree}").Trim()
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to read tree for $($project.project_id)"
    }
    Assert-Equal $actualTree $project.tree_sha "$($project.project_id) tree"

    $trackedFiles = @(& $GitPath -C $repositoryPath ls-files)
    if ($LASTEXITCODE -ne 0) {
        throw "Unable to list tracked files for $($project.project_id)"
    }
    Assert-Equal $trackedFiles.Count ([int]$project.tracked_file_count) "$($project.project_id) tracked file count"

    foreach ($file in $project.selected_files) {
        $relativePath = $file.path.Replace("/", [System.IO.Path]::DirectorySeparatorChar)
        $absolutePath = [System.IO.Path]::GetFullPath((Join-Path $repositoryPath $relativePath))
        if (-not $absolutePath.StartsWith($repositoryPath + [System.IO.Path]::DirectorySeparatorChar, [System.StringComparison]::OrdinalIgnoreCase)) {
            throw "Manifest path escapes repository: $($file.path)"
        }
        if (-not (Test-Path -LiteralPath $absolutePath -PathType Leaf)) {
            throw "Manifest file is missing: $absolutePath"
        }
        Assert-Equal (Get-Item -LiteralPath $absolutePath).Length ([long]$file.bytes) "$($project.project_id)/$($file.path) bytes"
        $actualHash = (Get-FileHash -LiteralPath $absolutePath -Algorithm SHA256).Hash.ToLowerInvariant()
        Assert-Equal $actualHash $file.sha256 "$($project.project_id)/$($file.path) SHA-256"
        $verifiedFiles += 1
    }

    $temporaryArchive = Join-Path ([System.IO.Path]::GetTempPath()) ("jiuheng-reference-" + [guid]::NewGuid().ToString("N") + ".tar")
    try {
        & $GitPath -C $repositoryPath archive --format=tar "--output=$temporaryArchive" $project.commit_sha
        if ($LASTEXITCODE -ne 0) {
            throw "Unable to create deterministic archive for $($project.project_id)"
        }
        Assert-Equal (Get-Item -LiteralPath $temporaryArchive).Length ([long]$project.archive.bytes) "$($project.project_id) archive bytes"
        $archiveHash = (Get-FileHash -LiteralPath $temporaryArchive -Algorithm SHA256).Hash.ToLowerInvariant()
        Assert-Equal $archiveHash $project.archive.sha256 "$($project.project_id) archive SHA-256"
    }
    finally {
        if (Test-Path -LiteralPath $temporaryArchive -PathType Leaf) {
            [System.IO.File]::Delete($temporaryArchive)
        }
    }

    $verifiedProjects += 1
}

Write-Output "Reference source verification passed: $verifiedProjects projects, $verifiedFiles selected files, commits, trees, and deterministic archives."
