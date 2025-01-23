# .\target\debug\onboarder-server.exe `
$port = 25569
if (Test-Path "port.txt") {
    $port = Get-Content "port.txt"
}
cargo run -- `
--port $port `
--notes-dir "C:\Users\TeamD\OneDrive\Documents\Ideas\YouTube\Notes" `
--downloads-dir "C:\Users\TeamD\OneDrive\Documents\Ideas\YouTube\Downloads" `
--search-dirs "C:\Users\TeamD\OneDrive\Documents\Ideas\YouTube\Downloads","C:\Users\TeamD\OneDrive\Videos\"
Pause
echo "bruh"
exit 1