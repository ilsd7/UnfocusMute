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

$Dist = Join-Path $RepoRoot "dist"
$PackageName = "UnfocusMute-$Version-windows-x64"
$LegacyStage = Join-Path $Dist $PackageName
$StageRoot = Join-Path $Dist ".package-$([System.Guid]::NewGuid().ToString('N'))"
$Stage = Join-Path $StageRoot $PackageName
$Zip = Join-Path $Dist "$PackageName.zip"
$TempZip = Join-Path $Dist "$PackageName.$PID.tmp.zip"
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)

function Convert-ReadmeForPackage {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Content
    )

    $Content = $Content.Replace('href="README.md"', 'href="README_ko.md"')
    $Content = $Content.Replace('href="../README.md"', 'href="../README_ko.md"')
    $Content = $Content.Replace('src="assets/app-icon.png"', 'src="assets/app-icon.ico"')
    $Content = $Content.Replace('src="../assets/app-icon.png"', 'src="../assets/app-icon.ico"')
    return $Content.TrimStart()
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
    $AssetsStage = Join-Path $Stage "assets"
    New-Item -ItemType Directory -Force -Path $AssetsStage | Out-Null
    Copy-Item "assets\app-icon.ico" $AssetsStage
    Copy-Item "assets\screenshot.png" $AssetsStage

    $ReadmeKo = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "README.md"), [System.Text.Encoding]::UTF8)
    $ReadmeKo = Convert-ReadmeForPackage $ReadmeKo
    [System.IO.File]::WriteAllText((Join-Path $Stage "README_ko.md"), $ReadmeKo, $Utf8NoBom)

    $ReadmeEn = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "README_en.md"), [System.Text.Encoding]::UTF8)
    $ReadmeEn = Convert-ReadmeForPackage $ReadmeEn
    [System.IO.File]::WriteAllText((Join-Path $Stage "README_en.md"), $ReadmeEn, $Utf8NoBom)

    if (Test-Path "docs" -PathType Container) {
        $DocsStage = Join-Path $Stage "docs"
        New-Item -ItemType Directory -Force -Path $DocsStage | Out-Null
        Get-ChildItem "docs" -File |
            Where-Object { $_.Name -ne "README.en.md" } |
            Sort-Object Name |
            ForEach-Object {
                $Readme = [System.IO.File]::ReadAllText($_.FullName, [System.Text.Encoding]::UTF8)
                $Readme = Convert-ReadmeForPackage $Readme
                [System.IO.File]::WriteAllText((Join-Path $DocsStage $_.Name), $Readme, $Utf8NoBom)
            }
    }

    if (Test-Path $TempZip) {
        Remove-Item $TempZip -Force
    }
    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $TempZip -CompressionLevel Optimal

    try {
        Replace-PackageZip $TempZip $Zip
    }
    catch {
        throw "Could not replace $Zip. Close File Explorer preview, archive tools, or any process using the existing ZIP, then try again. Original error: $($_.Exception.Message)"
    }

    if (Test-Path $LegacyStage) {
        Remove-Item $LegacyStage -Recurse -Force -ErrorAction SilentlyContinue
    }

    Write-Host "Created $Zip"
}
finally {
    if (Test-Path $TempZip) {
        Remove-Item $TempZip -Force -ErrorAction SilentlyContinue
    }
    if (Test-Path $StageRoot) {
        Remove-Item $StageRoot -Recurse -Force -ErrorAction SilentlyContinue
    }
    Pop-Location
}
