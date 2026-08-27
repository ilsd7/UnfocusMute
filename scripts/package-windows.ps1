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

function Replace-PackageOutputs {
    param(
        [Parameter(Mandatory = $true)]
        [string]$SourceZip,
        [Parameter(Mandatory = $true)]
        [string]$DestinationZip,
        [Parameter(Mandatory = $true)]
        [string]$SourceChecksum,
        [Parameter(Mandatory = $true)]
        [string]$DestinationChecksum,
        [Parameter(Mandatory = $true)]
        [string]$ZipFileName
    )

    $BackupId = [System.Guid]::NewGuid().ToString('N')
    $ZipBackup = "$DestinationZip.$BackupId.backup"
    $ChecksumBackup = "$DestinationChecksum.$BackupId.backup"
    $ZipBackedUp = $false
    $ChecksumBackedUp = $false
    $ZipInstalled = $false
    $ChecksumInstalled = $false
    try {
        if (Test-Path -LiteralPath $DestinationZip -PathType Leaf) {
            Move-Item -LiteralPath $DestinationZip -Destination $ZipBackup
            $ZipBackedUp = $true
        }
        if (Test-Path -LiteralPath $DestinationChecksum -PathType Leaf) {
            Move-Item -LiteralPath $DestinationChecksum -Destination $ChecksumBackup
            $ChecksumBackedUp = $true
        }

        Move-Item -LiteralPath $SourceZip -Destination $DestinationZip
        $ZipInstalled = $true
        Move-Item -LiteralPath $SourceChecksum -Destination $DestinationChecksum
        $ChecksumInstalled = $true

        Assert-ZipChecksum $DestinationZip $DestinationChecksum $ZipFileName
    }
    catch {
        $ReplaceError = $_
        $RollbackErrors = [System.Collections.Generic.List[string]]::new()
        try {
            if ($ZipInstalled -and (Test-Path -LiteralPath $DestinationZip -PathType Leaf)) {
                Remove-Item -LiteralPath $DestinationZip -Force
            }
            if ($ZipBackedUp) {
                Move-Item -LiteralPath $ZipBackup -Destination $DestinationZip
            }
        }
        catch {
            $RollbackErrors.Add("ZIP: $($_.Exception.Message)")
        }
        try {
            if ($ChecksumInstalled -and (Test-Path -LiteralPath $DestinationChecksum -PathType Leaf)) {
                Remove-Item -LiteralPath $DestinationChecksum -Force
            }
            if ($ChecksumBackedUp) {
                Move-Item -LiteralPath $ChecksumBackup -Destination $DestinationChecksum
            }
        }
        catch {
            $RollbackErrors.Add("checksum: $($_.Exception.Message)")
        }
        if ($RollbackErrors.Count -gt 0) {
            throw "Package replacement failed: $($ReplaceError.Exception.Message). Rollback also failed: $($RollbackErrors -join '; ')"
        }
        throw $ReplaceError
    }

    if ($ZipBackedUp) {
        Remove-Item -LiteralPath $ZipBackup -Force -ErrorAction SilentlyContinue
    }
    if ($ChecksumBackedUp) {
        Remove-Item -LiteralPath $ChecksumBackup -Force -ErrorAction SilentlyContinue
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
            $EntryName = $Entry.FullName
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

    if ($EntryName.Contains('\')) {
        throw "Package ZIP contains a non-canonical entry separator in $EntryName"
    }

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

function New-PackageZip {
    param(
        [Parameter(Mandatory = $true)]
        [string]$StagePath,
        [Parameter(Mandatory = $true)]
        [string]$PackageName,
        [Parameter(Mandatory = $true)]
        [string]$DestinationPath
    )

    # Windows PowerShell 5.1's Compress-Archive stores native backslashes in entry names.
    # Write each entry explicitly so the package remains portable across ZIP readers.
    Add-Type -AssemblyName System.IO.Compression
    Add-Type -AssemblyName System.IO.Compression.FileSystem

    $DestinationStream = [System.IO.File]::Open(
        $DestinationPath,
        [System.IO.FileMode]::CreateNew,
        [System.IO.FileAccess]::Write,
        [System.IO.FileShare]::None
    )
    try {
        $Archive = [System.IO.Compression.ZipArchive]::new(
            $DestinationStream,
            [System.IO.Compression.ZipArchiveMode]::Create,
            $true
        )
        try {
            foreach ($SourceFile in Get-ChildItem -LiteralPath $StagePath -File | Sort-Object -Property Name) {
                $EntryName = "$PackageName/$($SourceFile.Name)"
                Assert-ZipEntryName $EntryName
                $null = [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile(
                    $Archive,
                    $SourceFile.FullName,
                    $EntryName,
                    [System.IO.Compression.CompressionLevel]::Optimal
                )
            }
        }
        finally {
            $Archive.Dispose()
        }
    }
    finally {
        $DestinationStream.Dispose()
    }
}

function Remove-StalePackageTemps {
    param(
        [Parameter(Mandatory = $true)]
        [string]$DistPath,
        [Parameter(Mandatory = $true)]
        [string]$PackageName
    )

    if (-not (Test-Path $DistPath)) {
        return
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
    Remove-StalePackageTemps $Dist $PackageName

    cargo build --release --target $Target --locked
    if ($LASTEXITCODE -ne 0) {
        throw "cargo build failed with exit code $LASTEXITCODE"
    }

    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    Copy-Item "target\$Target\release\unfocusmute.exe" (Join-Path $Stage $ExeFileName)
    Copy-Item "LICENSE" $Stage
    Copy-Item "THIRD_PARTY_NOTICES.md" $Stage
    $RequiredEntries = @(
        "$PackageName/$ExeFileName",
        "$PackageName/LICENSE",
        "$PackageName/THIRD_PARTY_NOTICES.md"
    )

    New-PackageZip $Stage $PackageName $TempZip
    Assert-PackageZip $TempZip $RequiredEntries
    Write-ZipChecksum $TempZip $TempChecksum $ZipFileName
    Assert-ZipChecksum $TempZip $TempChecksum $ZipFileName

    try {
        Replace-PackageOutputs $TempZip $Zip $TempChecksum $Checksum $ZipFileName
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
