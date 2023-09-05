use std::path::PathBuf;

fn main() {
    let dir = PathBuf::from("./downloads");
    let abs = std::fs::canonicalize(dir).unwrap();
    let output = std::process::Command::new("pwsh")
        .current_dir(abs)
        .arg("-NoProfile")
        .arg("-WorkingDirectory")
        .arg("$(Get-Location)")
        .arg("-c")
        .arg(format!("wt pwsh.exe -NoProfile -WorkingDirectory $(Get-Location) -c 'echo hi && pwsh -WorkingDirectory $(Get-Location) && Write-Host \"\"press any key to close\"\" && $Host.UI.RawUI.ReadKey(\"\"NoEcho,IncludeKeyDown\"\")'"))
        .output()
        .expect("Failed to execute command");

    let res = if output.status.success() {
        println!("invoked successfully");
    } else {
        eprintln!("failed: {}", String::from_utf8_lossy(&output.stderr));
    };
}
