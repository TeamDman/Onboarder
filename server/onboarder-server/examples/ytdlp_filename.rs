fn get_ytdlp_filename(url: &str) -> Result<String, String> {
    let output = std::process::Command::new("yt-dlp")
        .arg("--print")
        .arg("filename")
        .arg("--cookies-from-browser")
        .arg("edge")
        .arg("--windows-filenames")
        .arg("--embed-metadata")
        .arg(url)
        .output()
        .expect("Failed to execute command");

    if output.status.success() {
        let fname = String::from_utf8_lossy(&output.stdout);
        let fname_trimmed = fname.trim_end_matches('\n');
        Ok(fname_trimmed.to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

fn main() {
    // Run the full command
    let url = "https://www.youtube.com/watch?v=EdYwBk2qe7Q";
    let fname = get_ytdlp_filename(url).unwrap();
    println!("Filename: \"{}\"", fname);
}
