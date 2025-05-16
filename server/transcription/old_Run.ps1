. .\Get-HuggingFaceToken.ps1
$files = Get-Content target_file.txt
foreach ($file in $files) {
    $file = $file.Replace("`"", "")
    Write-Host "Processing file: $file" -ForegroundColor Green
    uv run whisperx `
    --hf_token "$Env:HF_TOKEN" `
    --highlight_words True `
    --task "transcribe" `
    --language "en" `
    --output_dir "output" `
    --device "cuda" `
    --model "large-v2" `
    --min_speakers 0 `
    --max_speakers 1 `
    --diarize `
    $file | Out-Host
}

# --min_speakers 2 `
# --max_speakers 2 `