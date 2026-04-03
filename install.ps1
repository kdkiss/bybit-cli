param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]]$InstallerArgs
)

$repo = if ($env:BYBIT_CLI_REPO) { $env:BYBIT_CLI_REPO } else { "kdkiss/bybit-cli" }
$url = "https://github.com/$repo/releases/latest/download/bybit-cli-installer.ps1"
$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("bybit-cli-" + [System.Guid]::NewGuid().ToString("N"))
$installerPath = Join-Path $tempDir "bybit-cli-installer.ps1"

New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

try {
    Write-Host "Downloading generated installer from $url to $installerPath"
    Invoke-WebRequest -Uri $url -OutFile $installerPath
    & $installerPath @InstallerArgs
}
finally {
    Remove-Item -LiteralPath $tempDir -Recurse -Force -ErrorAction SilentlyContinue
}
