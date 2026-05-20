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

$Dist = Join-Path $RepoRoot "dist"
$PackageName = "UnfocusMute-$Version-$PackageSuffix"
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
        '(?ms)^\s*<div align="center">\s*<img src="(?:\.\./)?assets/app-icon\.png"[\s\S]*?</div>\s*',
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
        '(?ms)^[ \t]*<p align="center">\s*<img src="(?:\.\./)?assets/screenshot\.png"[^>]*>\s*</p>\s*',
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

function Write-ZipChecksum {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourceZip,
        [Parameter(Mandatory = $true)]
        [string]$Destination,
        [Parameter(Mandatory = $true)]
        [string]$ZipFileName
    )

    $Hash = (Get-FileHash -Algorithm SHA256 -Path $SourceZip).Hash.ToLowerInvariant()
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

    $ExpectedHash = (Get-FileHash -Algorithm SHA256 -Path $SourceZip).Hash.ToLowerInvariant()
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

        foreach ($Entry in $ZipFile.Entries) {
            if (-not $Entry.FullName.EndsWith(".md", [System.StringComparison]::OrdinalIgnoreCase)) {
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
            if ($Content.Contains('src="assets/screenshot.png"') -or $Content.Contains('src="../assets/screenshot.png"')) {
                throw "Package README $($Entry.FullName) still references screenshot.png"
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

Push-Location $RepoRoot
try {
    cargo build --release --target $Target --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    Copy-Item "target\$Target\release\unfocusmute.exe" (Join-Path $Stage "UnfocusMute.exe")
    Copy-Item "LICENSE" $Stage
    Copy-Item "THIRD_PARTY_NOTICES.md" $Stage
    $DocFiles = @()
    if (Test-Path "docs" -PathType Container) {
        $DocFiles = @(
            Get-ChildItem "docs" -File -Filter "*.md" |
                Where-Object { $_.Name -ne "README.en.md" } |
                Sort-Object Name
        )
    }

    $ReadmeKo = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "README.md"), [System.Text.Encoding]::UTF8)
    $ReadmeKo = Convert-ReadmeForPackage $ReadmeKo
    [System.IO.File]::WriteAllText((Join-Path $Stage "README_ko.md"), $ReadmeKo, $Utf8NoBom)

    $ReadmeEn = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "README_en.md"), [System.Text.Encoding]::UTF8)
    $ReadmeEn = Convert-ReadmeForPackage $ReadmeEn
    [System.IO.File]::WriteAllText((Join-Path $Stage "README_en.md"), $ReadmeEn, $Utf8NoBom)

    if ($DocFiles.Count -gt 0) {
        $DocsStage = Join-Path $Stage "docs"
        New-Item -ItemType Directory -Force -Path $DocsStage | Out-Null
        foreach ($DocFile in $DocFiles) {
            $Readme = [System.IO.File]::ReadAllText($DocFile.FullName, [System.Text.Encoding]::UTF8)
            $Readme = Convert-ReadmeForPackage $Readme
            [System.IO.File]::WriteAllText((Join-Path $DocsStage $DocFile.Name), $Readme, $Utf8NoBom)
        }
    }
    $RequiredEntries = @(
        "UnfocusMute.exe",
        "LICENSE",
        "THIRD_PARTY_NOTICES.md",
        "README_ko.md",
        "README_en.md"
    )
    if ($DocFiles.Count -gt 0) {
        $RequiredEntries += $DocFiles | ForEach-Object { "docs/$($_.Name)" }
    }

    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $TempZip -CompressionLevel Optimal
    Assert-PackageZip $TempZip $RequiredEntries
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
