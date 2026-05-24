param(
    [string]$Target = "x86_64-pc-windows-msvc",
    [string]$ReleaseTag = ""
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$CargoToml = Join-Path $RepoRoot "Cargo.toml"
$VersionLine = Select-String -Path $CargoToml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $VersionLine) {
    throw "Could not read package version from Cargo.toml"
}

$Version = $VersionLine.Matches[0].Groups[1].Value
$ResolvedReleaseTag = $ReleaseTag
if (-not $ResolvedReleaseTag -and $env:GITHUB_REF_TYPE -eq "tag") {
    $ResolvedReleaseTag = $env:GITHUB_REF_NAME
}
if ($ResolvedReleaseTag) {
    $ExpectedTag = "v$Version"
    if ($ResolvedReleaseTag -ne $ExpectedTag) {
        throw "Release tag $ResolvedReleaseTag does not match Cargo.toml version $Version. Expected $ExpectedTag."
    }
}

$PackageSuffixByTarget = [System.Collections.Generic.Dictionary[string, string]]::new([System.StringComparer]::Ordinal)
$PackageSuffixByTarget.Add("x86_64-pc-windows-msvc", "windows-x64")
$PackageSuffixByTarget.Add("aarch64-pc-windows-msvc", "windows-arm64")
$PackageSuffixByTarget.Add("i686-pc-windows-msvc", "windows-x86")
if (-not $PackageSuffixByTarget.ContainsKey($Target)) {
    $SupportedTargets = ($PackageSuffixByTarget.Keys | Sort-Object) -join ", "
    throw "Unsupported package target $Target. Supported targets: $SupportedTargets."
}
$PackageSuffix = $PackageSuffixByTarget[$Target]
$ExeFileName = "UnfocusMute-v$Version.exe"

$Dist = Join-Path $RepoRoot "dist"
$PackageName = "UnfocusMute-$PackageSuffix"
$LegacyStage = Join-Path $Dist $PackageName
$StageRoot = Join-Path $Dist ".package-$([System.Guid]::NewGuid().ToString('N'))"
$Stage = Join-Path $StageRoot $PackageName
$Zip = Join-Path $Dist "$PackageName.zip"
$ZipFileName = [System.IO.Path]::GetFileName($Zip)
$TempZip = Join-Path $Dist "$PackageName.$([System.Guid]::NewGuid().ToString('N')).tmp.zip"
$Checksum = Join-Path $Dist "$PackageName.zip.sha256"
$TempChecksum = Join-Path $Dist "$PackageName.$([System.Guid]::NewGuid().ToString('N')).tmp.zip.sha256"
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)

function Convert-ReadmeForPackage {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    $Content = Remove-ReadmeHeaderBlock $Content
    $Content = Remove-ReadmeLanguageLinks $Content
    $Content = Remove-ReadmeScreenshotBlock $Content
    $Content = Remove-ReadmeLatestDownloadTable $Content
    return $Content.TrimStart()
}

function Remove-ReadmeLanguageLinks {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    return [System.Text.RegularExpressions.Regex]::Replace(
        $Content,
        '(?m)^[ \t]*(?=[^\r\n]*\|)(?=[^\r\n]*README)(?=[^\r\n]*\.md)[^\r\n]*(?:\r?\n[ \t]*\r?\n|\r?\n|$)',
        ''
    )
}

function Remove-ReadmeHeaderBlock {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    return [System.Text.RegularExpressions.Regex]::Replace(
        $Content,
        '(?ms)^\s*<div align="center">\s*<img src="(?:\.\./)?assets/app-icon\.png"[\s\S]*?</div>\s*(?:---\s*)?',
        "# UnfocusMute`r`n`r`n"
    )
}

function Remove-ReadmeScreenshotBlock {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    return [System.Text.RegularExpressions.Regex]::Replace(
        $Content,
        '(?ms)^[ \t]*<p align="center">\s*<img src="(?:\.\./)?assets/screenshot(?:_[A-Za-z0-9-]+)?\.png"[^>]*>\s*</p>\s*',
        ''
    )
}

function Remove-ReadmeLatestDownloadTable {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    return [System.Text.RegularExpressions.Regex]::Replace(
        $Content,
        '(?m)^[ \t]*\|[^\r\n]*\|\r?\n[ \t]*\|[ \t]*---[ \t]*\|\r?\n[ \t]*\|[^\r\n]*releases/latest/download/UnfocusMute-windows-x64\.zip[^\r\n]*\|\r?\n[ \t]*\|[^\r\n]*releases/latest[^\r\n]*\|\r?\n(?:[ \t]*\r?\n)?',
        ''
    )
}

function Replace-PackageZip {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Source,
        [Parameter(Mandatory = $true)]
        [string]$Destination
    )

    if (Test-Path $Destination -PathType Leaf) {
        [System.IO.File]::Replace(
            [System.IO.Path]::GetFullPath($Source),
            [System.IO.Path]::GetFullPath($Destination),
            $null
        )
    }
    else {
        Move-Item $Source $Destination
    }
}

function Get-Sha256Hash {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    if (Get-Command Get-FileHash -ErrorAction SilentlyContinue) {
        return (Get-FileHash -Algorithm SHA256 -Path $Path).Hash.ToLowerInvariant()
    }

    $Stream = [System.IO.File]::OpenRead($Path)
    try {
        $Sha256 = [System.Security.Cryptography.SHA256]::Create()
        try {
            $HashBytes = $Sha256.ComputeHash($Stream)
        }
        finally {
            if ($null -ne $Sha256) {
                $Sha256.Dispose()
            }
        }
    }
    finally {
        $Stream.Dispose()
    }

    return ([System.BitConverter]::ToString($HashBytes)).Replace("-", "").ToLowerInvariant()
}

function Write-ZipChecksum {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourceZip,
        [Parameter(Mandatory = $true)]
        [string]$Destination,
        [Parameter(Mandatory = $true)]
        [string]$ZipFileName
    )

    $Hash = Get-Sha256Hash $SourceZip
    [System.IO.File]::WriteAllText($Destination, "$Hash  $ZipFileName`n", $Utf8NoBom)
}

function Assert-ZipChecksum {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourceZip,
        [Parameter(Mandatory = $true)]
        [string]$ChecksumPath,
        [Parameter(Mandatory = $true)]
        [string]$ZipFileName
    )

    $Lines = [System.IO.File]::ReadAllLines($ChecksumPath, [System.Text.Encoding]::UTF8)
    if ($Lines.Count -ne 1) {
        throw "Package checksum file must contain exactly one line"
    }

    $Parts = $Lines[0].Trim() -split '\s+'
    if ($Parts.Count -ne 2) {
        throw "Package checksum file must contain a SHA-256 hash and ZIP file name"
    }
    if ($Parts[0] -cnotmatch '^[0-9a-f]{64}$') {
        throw "Package checksum hash must be 64 lowercase hexadecimal characters"
    }

    $ExpectedHash = Get-Sha256Hash $SourceZip
    if ($Parts[0] -cne $ExpectedHash) {
        throw "Package checksum hash does not match $ZipFileName"
    }
    if ($Parts[1] -cne $ZipFileName) {
        throw "Package checksum file name does not match $ZipFileName"
    }
}

function Assert-PackageZip {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$ExpectedExeFileName,
        [Parameter(Mandatory = $true)]
        [string[]]$RequiredEntries
    )

    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $ZipFile = [System.IO.Compression.ZipFile]::OpenRead($Path)
    try {
        $Entries = [System.Collections.Generic.Dictionary[string, System.IO.Compression.ZipArchiveEntry]]::new([System.StringComparer]::Ordinal)
        $WindowsEntryNames = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
        $ExpectedEntries = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::Ordinal)
        foreach ($RequiredEntry in $RequiredEntries) {
            $null = $ExpectedEntries.Add($RequiredEntry)
        }
        foreach ($Entry in $ZipFile.Entries) {
            $EntryName = $Entry.FullName.Replace('\', '/')
            if ($EntryName.EndsWith('/')) {
                continue
            }
            Assert-ZipEntryName $EntryName
            if ([System.Text.RegularExpressions.Regex]::IsMatch($EntryName, '(^|/)README[^/]*\.md$', [System.Text.RegularExpressions.RegexOptions]::IgnoreCase)) {
                throw "Package README must use .txt extension, found $EntryName"
            }
            if ($Entries.ContainsKey($EntryName)) {
                throw "Package ZIP contains duplicate entry $EntryName"
            }
            if (-not $WindowsEntryNames.Add($EntryName)) {
                throw "Package ZIP contains a case-insensitive duplicate entry $EntryName"
            }
            $Entries.Add($EntryName, $Entry)
        }
        foreach ($RequiredEntry in $RequiredEntries) {
            if (-not $Entries.ContainsKey($RequiredEntry)) {
                throw "Package ZIP is missing $RequiredEntry"
            }
            if ($Entries[$RequiredEntry].Length -eq 0) {
                throw "Package ZIP entry $RequiredEntry is empty"
            }
        }
        foreach ($EntryName in $Entries.Keys) {
            if (-not $ExpectedEntries.Contains($EntryName)) {
                throw "Package ZIP contains unexpected entry $EntryName"
            }
        }

        if (-not $Entries.ContainsKey($ExpectedExeFileName)) {
            throw "Package ZIP is missing versioned executable $ExpectedExeFileName"
        }

        foreach ($Entry in $ZipFile.Entries) {
            if (-not (
                $Entry.FullName.EndsWith(".md", [System.StringComparison]::OrdinalIgnoreCase) -or
                $Entry.FullName.EndsWith(".txt", [System.StringComparison]::OrdinalIgnoreCase)
            )) {
                continue
            }
            $Reader = [System.IO.StreamReader]::new($Entry.Open(), [System.Text.Encoding]::UTF8, $true)
            try {
                $Content = $Reader.ReadToEnd()
            }
            finally {
                $Reader.Dispose()
            }

            if ($Content.Contains('src="assets/app-icon.png"') -or $Content.Contains('src="../assets/app-icon.png"')) {
                throw "Package README $($Entry.FullName) still references app-icon.png"
            }
            if ([System.Text.RegularExpressions.Regex]::IsMatch($Content, 'src="(?:\.\./)?assets/screenshot(?:_[A-Za-z0-9-]+)?\.png"')) {
                throw "Package README $($Entry.FullName) still references a screenshot image"
            }
            if ($Content.Contains("releases/latest/download/UnfocusMute-windows-x64.zip")) {
                throw "Package README $($Entry.FullName) still references the latest ZIP download"
            }
        }
    }
    finally {
        $ZipFile.Dispose()
    }
}

function Assert-ZipEntryName {
    param(
        [Parameter(Mandatory = $true)]
        [string]$EntryName
    )

    if ([System.IO.Path]::IsPathRooted($EntryName)) {
        throw "Package ZIP contains absolute entry $EntryName"
    }

    $Segments = $EntryName -split '/'
    foreach ($Segment in $Segments) {
        if ($Segment.Length -eq 0 -or $Segment -eq "." -or $Segment -eq "..") {
            throw "Package ZIP contains unsafe entry $EntryName"
        }
    }
}

function Remove-PackageOutputs {
    param(
        [Parameter(Mandatory = $true)]
        [string]$DistPath,
        [Parameter(Mandatory = $true)]
        [string]$PackageName
    )

    if (-not (Test-Path $DistPath)) {
        return
    }

    $OutputPaths = @(
        (Join-Path $DistPath $PackageName)
    )
    foreach ($OutputPath in $OutputPaths) {
        if (Test-Path $OutputPath) {
            Remove-Item $OutputPath -Recurse -Force
        }
    }

    Get-ChildItem $DistPath -Force |
        Where-Object {
            $_.Name -like ".package-*" -or
            $_.Name -like "$PackageName.*.tmp.zip" -or
            $_.Name -like "$PackageName.*.tmp.zip.sha256"
        } |
        Remove-Item -Recurse -Force
}

Push-Location $RepoRoot
try {
    Remove-PackageOutputs $Dist $PackageName

    cargo build --release --target $Target --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    Copy-Item "target\$Target\release\unfocusmute.exe" (Join-Path $Stage $ExeFileName)
    Copy-Item "LICENSE" $Stage
    Copy-Item "THIRD_PARTY_NOTICES.md" $Stage
    $DocFiles = @()
    if (Test-Path "docs" -PathType Container) {
        $DocFiles = @(
            Get-ChildItem "docs" -File -Filter "README_*.md" |
                Where-Object { $_.Name -ne "README_ko.md" } |
                Sort-Object Name
        )
    }
    $DocsStage = Join-Path $Stage "docs"
    New-Item -ItemType Directory -Force -Path $DocsStage | Out-Null

    $ReadmeKo = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "docs\README_ko.md"), [System.Text.Encoding]::UTF8)
    $ReadmeKo = Convert-ReadmeForPackage $ReadmeKo
    [System.IO.File]::WriteAllText((Join-Path $DocsStage "README_ko.txt"), $ReadmeKo, $Utf8NoBom)

    if ($DocFiles.Count -gt 0) {
        foreach ($DocFile in $DocFiles) {
            $Readme = [System.IO.File]::ReadAllText($DocFile.FullName, [System.Text.Encoding]::UTF8)
            $Readme = Convert-ReadmeForPackage $Readme
            $DocTextName = [System.IO.Path]::ChangeExtension($DocFile.Name, ".txt")
            [System.IO.File]::WriteAllText((Join-Path $DocsStage $DocTextName), $Readme, $Utf8NoBom)
        }
    }
    $ExpectedExeEntry = "$PackageName/$ExeFileName"
    $RequiredEntries = @(
        $ExpectedExeEntry,
        "$PackageName/LICENSE",
        "$PackageName/THIRD_PARTY_NOTICES.md",
        "$PackageName/docs/README_ko.txt"
    )
    if ($DocFiles.Count -gt 0) {
        $RequiredEntries += $DocFiles | ForEach-Object { "$PackageName/docs/$([System.IO.Path]::ChangeExtension($_.Name, '.txt'))" }
    }

    Compress-Archive -Path $Stage -DestinationPath $TempZip -CompressionLevel Optimal
    Assert-PackageZip $TempZip $ExpectedExeEntry $RequiredEntries
    Write-ZipChecksum $TempZip $TempChecksum $ZipFileName
    Assert-ZipChecksum $TempZip $TempChecksum $ZipFileName

    try {
        Replace-PackageZip $TempZip $Zip
        Replace-PackageZip $TempChecksum $Checksum
        Assert-ZipChecksum $Zip $Checksum $ZipFileName
    }
    catch {
        throw "Could not replace package outputs. Close File Explorer preview, archive tools, or any process using the existing ZIP/checksum, then try again. Original error: $($_.Exception.Message)"
    }

    if (Test-Path $LegacyStage) {
        Remove-Item $LegacyStage -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "Created $Zip"
    Write-Host "Created $Checksum"
}
finally {
    if (Test-Path $TempZip) {
        Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $TempChecksum) {
        Remove-Item $TempChecksum -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $StageRoot) {
        Remove-Item $StageRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
    Pop-Location
}
