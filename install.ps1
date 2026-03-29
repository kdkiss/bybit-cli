$repo = if ($env:BYBIT_CLI_REPO) { $env:BYBIT_CLI_REPO } else { "kdkiss/bybit-cli" }
$url = "https://github.com/$repo/releases/latest/download/bybit-cli-installer.ps1"

Write-Host "Fetching generated installer from $url"
Invoke-Expression (Invoke-RestMethod -Uri $url)
