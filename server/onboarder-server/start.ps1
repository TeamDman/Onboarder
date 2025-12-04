param(
    [switch]$cargo
)

$ErrorActionPreference = "Stop"



$port = 25569
if (Test-Path "port.txt") {
    $port = Get-Content "port.txt"
}

$notesDir = "C:\Users\TeamD\OneDrive\Documents\Ideas\YouTube\Notes"
$downloadsDir = "C:\Users\TeamD\OneDrive\Documents\Ideas\YouTube\Downloads"
$searchDirs = "C:\Users\TeamD\OneDrive\Documents\Ideas\YouTube\Downloads,C:\Users\TeamD\OneDrive\Videos\"

$exeArgs = @(
    "--port", $port,
    "--notes-dir", $notesDir,
    "--downloads-dir", $downloadsDir,
    "--search-dirs", $searchDirs
)

if ($cargo) {
    $cargoArgs = @("run", "--") + $exeArgs
    & cargo @cargoArgs
} else {
    & onboarder-server @exeArgs
}

Pause
Write-Output "bruh"
exit 1
