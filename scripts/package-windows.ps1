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
$Stage = Join-Path $Dist "UnfocusMute-$Version-windows-x64"
$Zip = "$Stage.zip"

Push-Location $RepoRoot
try {
    cargo build --release --target $Target

    if (Test-Path $Stage) {
        Remove-Item $Stage -Recurse -Force
    }
    New-Item -ItemType Directory -Force -Path $Stage | Out-Null

    Copy-Item "target\$Target\release\unfocusmute.exe" (Join-Path $Stage "UnfocusMute.exe")
    Copy-Item "README.md" $Stage
    Copy-Item "LICENSE" $Stage
    Copy-Item "icon.png" $Stage

    if (Test-Path $Zip) {
        Remove-Item $Zip -Force
    }
    Compress-Archive -Path (Join-Path $Stage "*") -DestinationPath $Zip
    Write-Host "Created $Zip"
}
finally {
    Pop-Location
}
