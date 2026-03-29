# bybit-cli Windows Installer
$repo = if ($env:BYBIT_CLI_REPO) { $env:BYBIT_CLI_REPO } else { "kdkiss/bybit-cli" }
$bin = "bybit.exe"
$installDir = "$HOME\.local\bin"

# Ensure install directory exists
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
}

# Get latest release tag
try {
    $latest = (Invoke-RestMethod -Uri "https://api.github.io/repos/$repo/releases/latest").tag_name
} catch {
    Write-Error "Could not determine latest release. Check https://github.com/$repo/releases"
    exit 1
}

$url = "https://github.com/$repo/releases/download/$latest/bybit-windows-x64.exe"

Write-Host "Installing bybit-cli $latest (windows-x64)..."
Write-Host "Downloading: $url"

$tempPath = [System.IO.Path]::GetTempFileName()
try {
    Invoke-WebRequest -Uri $url -OutFile $tempPath
    Move-Item -Path $tempPath -Destination "$installDir\$bin" -Force
} finally {
    if (Test-Path $tempPath) { Remove-Item $tempPath }
}

Write-Host ""
Write-Host "Installed to: $installDir\$bin"

# Check if PATH contains the install directory
$path = [Environment]::GetEnvironmentVariable("Path", "User")
if ($path -notlike "*$installDir*") {
    Write-Host ""
    Write-Host "NOTE: $installDir is not in your PATH."
    Write-Host "To add it, run:"
    Write-Host "  [Environment]::SetEnvironmentVariable('Path', `$env:Path + ';$installDir', 'User')"
    Write-Host "Then restart your terminal."
}

Write-Host ""
Write-Host "Run 'bybit --help' to get started."
Write-Host "Run 'bybit setup' to configure your API credentials."
