async fn get_ytdlp_filename(url: &str) -> Result<String, String> {
    let output = tokio::process::Command::new("yt-dlp")
        .arg("--encoding")
        .arg("utf-8")
        .arg("--print")
        .arg("filename")
        .arg("--cookies-from-browser")
        .arg("edge")
        .arg("--windows-filenames")
        .arg("--embed-metadata")
        .arg(url)
        .output()
        .await
        .expect("Failed to execute command");

    if output.status.success() {
        let fname = String::from_utf8_lossy(&output.stdout);
        let fname_trimmed = fname.trim_end_matches('\n');
        Ok(fname_trimmed.to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tokio::main]
async fn main() {
    // Run the full command
    let url = "https://www.youtube.com/watch?v=-TVw_ndGyW4";
    let fname = get_ytdlp_filename(url).await.unwrap();
    println!("Filename: \"{}\"", fname);
}
