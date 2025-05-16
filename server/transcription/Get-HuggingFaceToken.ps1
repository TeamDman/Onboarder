if (-not (Test-Path -Path Env:\HF_TOKEN)) {
    Write-Host "[Get-HuggingFaceToken] Make sure to dot-source this file!"
    $password = op read "op://Private/o24pfzdtppu4asfopqhzya5rg4/credential" --no-newline
    $env:HF_TOKEN = $password
}