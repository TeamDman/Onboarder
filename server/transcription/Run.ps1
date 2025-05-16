param (
    [int]$port = 5899,
    [string]$api_key
)

if (-not $api_key) {
    $api_key = Read-Host -Prompt "Enter your secret key" -MaskInput
}

Write-Host "Starting server!"
uv run main.py $port $api_key