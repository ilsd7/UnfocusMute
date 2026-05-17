param(
    [string]$Target = "x86_64-pc-windows-msvc"
)

$ErrorActionPreference = "Stop"

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$CargoToml = Join-Path $RepoRoot "Cargo.toml"
$VersionLine = Select-String -Path $CargoToml -Pattern '^version\s*=\s*"([^"]+)"' | Select-Object -First 1
if (-not $VersionLine) {
    throw "Could not read package version from Cargo.toml"
}

$Version = $VersionLine.Matches[0].Groups[1].Value
$Dist = Join-Path $RepoRoot "dist"
$PackageName = "UnfocusMute-$Version-windows-x64"
$LegacyStage = Join-Path $Dist $PackageName
$StageRoot = Join-Path $Dist ".package-$([System.Guid]::NewGuid().ToString('N'))"
$Stage = Join-Path $StageRoot $PackageName
$Zip = Join-Path $Dist "$PackageName.zip"
$TempZip = Join-Path $Dist "$PackageName.$PID.tmp.zip"
$Utf8NoBom = [System.Text.UTF8Encoding]::new($false)

Push-Location $RepoRoot
try {
    cargo build --release --target $Target

    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    Copy-Item "target\$Target\release\unfocusmute.exe" (Join-Path $Stage "UnfocusMute.exe")
    Copy-Item "LICENSE" $Stage

    $ReadmeKo = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "README.md"), [System.Text.Encoding]::UTF8)
    [System.IO.File]::WriteAllText((Join-Path $Stage "README_ko.md"), $ReadmeKo, $Utf8NoBom)

    $ReadmeEn = [System.IO.File]::ReadAllText((Join-Path $RepoRoot "README_en.md"), [System.Text.Encoding]::UTF8)
    $ReadmeEn = $ReadmeEn.Replace("[한국어](README.md)", "[한국어](README_ko.md)")
    [System.IO.File]::WriteAllText((Join-Path $Stage "README_en.md"), $ReadmeEn, $Utf8NoBom)

    if (Test-Path "assets\app-icon.png") {
        New-Item -ItemType Directory -Force -Path (Join-Path $Stage "assets") | Out-Null
        Copy-Item "assets\app-icon.png" (Join-Path $Stage "assets\app-icon.png")
    }
    if (Test-Path "docs") {
        $DocsStage = Join-Path $Stage "docs"
        New-Item -ItemType Directory -Force -Path $DocsStage | Out-Null
        Get-ChildItem "docs" -File |
            Where-Object { $_.Name -ne "README.en.md" } |
            Copy-Item -Destination $DocsStage
    }

    if (Test-Path $TempZip) {
        Remove-Item $TempZip -Force
    }
    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $TempZip

    try {
        if (Test-Path $Zip) {
            Remove-Item $Zip -Force
        }
        Move-Item $TempZip $Zip
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
