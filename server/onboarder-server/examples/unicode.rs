#[tokio::main]
async fn main() {
    println!("ch");
    let ch = "｜";
    println!("\"{}\" len={}", ch, ch.len());
    println!("");
    // ch
    // "｜" len=3

    println!("bytes");
    let bytes = ch.as_bytes();
    println!("\"{:?}\" len={}", bytes, bytes.len());
    println!("");
    // bytes
    // "[239, 189, 156]" len=3

    println!("decoded");
    let decoded = String::from_utf8_lossy(bytes);
    println!("\"{}\" len={}", decoded, decoded.len());
    println!("\"{}\" == \"{}\" ? {}", ch, decoded, ch == decoded);
    println!("");
    // "｜" len=3
    // "｜" == "｜" ? true

    println!("proc");
    let output = std::process::Command::new("target/debug/examples/unicode_echo.exe")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "｜" len=3
    // "｜" == "｜" ? true

    println!("tokio");
    let output = tokio::process::Command::new("target/debug/examples/unicode_echo.exe")
        .output()
        .await
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "｜" len=3
    // "｜" == "｜" ? true

    println!("python1");
    let output = std::process::Command::new(r"C:\ProgramData\Anaconda3\python.exe")
        .arg("-c")
        .arg("print('｜', end='')")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "" len=0
    // "｜" == "" ? false

    println!("python2");
    let output = std::process::Command::new(r"C:\ProgramData\Anaconda3\python.exe")
        .arg("-c")
        .arg("import sys; print(sys.stdout.encoding, end='')")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "cp1252" len=6
    // "｜" == "cp1252" ? false

    println!("python3");
    let output = std::process::Command::new(r"C:\ProgramData\Anaconda3\python.exe")
        .arg("-c")
        .arg("import sys; print(sys.stdout.encoding, end='')")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "utf-8" len=5
    // "｜" == "utf-8" ? false

    println!("python4");
    let output = std::process::Command::new(r"C:\ProgramData\Anaconda3\python.exe")
        .arg("-c")
        .arg("print('｜', end='')")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "｜" len=3
    // "｜" == "｜" ? true

    println!("pwsh1");
    let output = tokio::process::Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("Write-Host -NoNewLine \"｜\"")
        .output()
        .await
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "|" len=1
    // "｜" == "|" ? false

    println!("pwsh2");
    let output = tokio::process::Command::new("pwsh")
    .arg("-NoProfile")
    .arg("-Command")
    .arg("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; Write-Host -NoNewLine \"｜\"")
    .output()
        .await
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "｜" len=3
    // "｜" == "｜" ? true

    println!("pwsh3");
    let output = tokio::process::Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("Write-Host -NoNewLine \"｜\"")
        .output()
        .await
        .expect("Failed to execute command");
    let stdout = String::from_utf8_lossy(&output.stdout);
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "｜" len=3
    // "｜" == "｜" ? true

    println!("ytdlp1");
    let output = std::process::Command::new("yt-dlp")
        .arg("--print")
        .arg("filename")
        .arg("--windows-filenames")
        .arg("https://www.youtube.com/watch?v=-TVw_ndGyW4")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "solivagant  a breakcore mix [-TVw_ndGyW4].webm " len=47
    // "｜" == "solivagant  a breakcore mix [-TVw_ndGyW4].webm " ? false

    println!("ytdlp2");
    let output = std::process::Command::new("yt-dlp")
        .arg("--print")
        .arg("filename")
        .arg("https://www.youtube.com/watch?v=-TVw_ndGyW4")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "solivagant  a breakcore mix [-TVw_ndGyW4].webm " len=47
    // "｜" == "solivagant  a breakcore mix [-TVw_ndGyW4].webm " ? false

    println!("ytdlp3");
    let output = std::process::Command::new("pwsh")
        .arg("-NoProfile")
        .arg("-Command")
        .arg("[Console]::OutputEncoding = [System.Text.Encoding]::UTF8; yt-dlp --print filename --windows-filenames https://www.youtube.com/watch?v=-TVw_ndGyW4")
        .env("PYTHONIOENCODING","utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "solivagant  a breakcore mix [-TVw_ndGyW4].webm " len=47
    // "｜" == "solivagant  a breakcore mix [-TVw_ndGyW4].webm " ? false

    println!("ytdlp4");
    let output = std::process::Command::new("yt-dlp")
        .arg("-vU")
        .arg("--print")
        .arg("filename")
        .arg("https://www.youtube.com/watch?v=-TVw_ndGyW4")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // stderr: [debug] Command-line config: ['-vU', '--print', 'filename', 'https://www.youtube.com/watch?v=-TVw_ndGyW4']
    // [debug] Encodings: locale cp1252, fs utf-8, pref cp1252, out cp1252 (No VT), error cp1252 (No VT), screen cp1252 (No VT)
    // [debug] yt-dlp version stable@2023.07.06 [b532a3481] (win_exe)
    // [debug] Python 3.8.10 (CPython AMD64 64bit) - Windows-10-10.0.19045-SP0 (OpenSSL 1.1.1k  25 Mar 2021)
    // [debug] exe versions: ffmpeg 4.3.1-2020-11-19-full_build-www.gyan.dev, ffprobe 4.3.1-2020-11-19-full_build-www.gyan.dev
    // [debug] Optional libraries: Cryptodome-3.18.0, brotli-1.0.9, certifi-2023.05.07, mutagen-1.46.0, sqlite3-2.6.0, websockets-11.0.3
    // [debug] Proxy map: {}
    // [debug] Loaded 1855 extractors
    // [debug] Fetching release info: https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest
    // Available version: stable@2023.07.06, Current version: stable@2023.07.06
    // Current Build Hash: 5ff3e702171a50175c34397494e2d18ce35d771c2110b1e59bd173ec2fb352aa
    // yt-dlp is up to date (stable@2023.07.06)
    // [youtube] Extracting URL: https://www.youtube.com/watch?v=-TVw_ndGyW4
    // [youtube] -TVw_ndGyW4: Downloading webpage
    // [youtube] -TVw_ndGyW4: Downloading ios player API JSON
    // [youtube] -TVw_ndGyW4: Downloading android player API JSON
    // [youtube] -TVw_ndGyW4: Downloading m3u8 information
    // [debug] Sort order given by extractor: quality, res, fps, hdr:12, source, vcodec:vp9.2, channels, acodec, lang, proto
    // [debug] Formats sorted by: hasvid, ie_pref, quality, res, fps, hdr:12(7), source, vcodec:vp9.2(10), channels, acodec, lang, proto, size, br, asr, vext, aext, hasaud, id
    // [debug] Default format spec: bestvideo*+bestaudio/best
    // [info] -TVw_ndGyW4: Downloading 1 format(s): 248+251

    // "solivagant  a breakcore mix [-TVw_ndGyW4].webm
    // " len=47
    // "｜" == "solivagant  a breakcore mix [-TVw_ndGyW4].webm
    // " ? false

    println!("ytdlp5");
    let output = std::process::Command::new("yt-dlp")
        .arg("-vU")
        .arg("--encoding")
        .arg("utf-8")
        .arg("--print")
        .arg("filename")
        .arg("https://www.youtube.com/watch?v=-TVw_ndGyW4")
        .env("PYTHONIOENCODING", "utf-8")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // stderr: [debug] Command-line config: ['-vU', '--encoding', 'utf-8', '--print', 'filename', 'https://www.youtube.com/watch?v=-TVw_ndGyW4']
    // [debug] Encodings: locale cp1252, fs utf-8, pref utf-8, out cp1252 (No VT), error cp1252 (No VT), screen cp1252 (No VT)
    // [debug] yt-dlp version stable@2023.07.06 [b532a3481] (win_exe)
    // [debug] Python 3.8.10 (CPython AMD64 64bit) - Windows-10-10.0.19045-SP0 (OpenSSL 1.1.1k  25 Mar 2021)
    // [debug] exe versions: ffmpeg 4.3.1-2020-11-19-full_build-www.gyan.dev, ffprobe 4.3.1-2020-11-19-full_build-www.gyan.dev
    // [debug] Optional libraries: Cryptodome-3.18.0, brotli-1.0.9, certifi-2023.05.07, mutagen-1.46.0, sqlite3-2.6.0, websockets-11.0.3
    // [debug] Proxy map: {}
    // [debug] Loaded 1855 extractors
    // [debug] Fetching release info: https://api.github.com/repos/yt-dlp/yt-dlp/releases/latest
    // Available version: stable@2023.07.06, Current version: stable@2023.07.06
    // Current Build Hash: 5ff3e702171a50175c34397494e2d18ce35d771c2110b1e59bd173ec2fb352aa
    // yt-dlp is up to date (stable@2023.07.06)
    // [youtube] Extracting URL: https://www.youtube.com/watch?v=-TVw_ndGyW4
    // [youtube] -TVw_ndGyW4: Downloading webpage
    // [youtube] -TVw_ndGyW4: Downloading ios player API JSON
    // [youtube] -TVw_ndGyW4: Downloading android player API JSON
    // [youtube] -TVw_ndGyW4: Downloading m3u8 information
    // [debug] Sort order given by extractor: quality, res, fps, hdr:12, source, vcodec:vp9.2, channels, acodec, lang, proto
    // [debug] Formats sorted by: hasvid, ie_pref, quality, res, fps, hdr:12(7), source, vcodec:vp9.2(10), channels, acodec, lang, proto, size, br, asr, vext, aext, hasaud, id
    // [debug] Default format spec: bestvideo*+bestaudio/best
    // [info] -TVw_ndGyW4: Downloading 1 format(s): 248+251

    // "solivagant ｜ a breakcore mix [-TVw_ndGyW4].webm
    // " len=50
    // "｜" == "solivagant ｜ a breakcore mix [-TVw_ndGyW4].webm
    // " ? false

    println!("ytdlp6");
    let output = std::process::Command::new("yt-dlp")
        .arg("-vU")
        .arg("--encoding")
        .arg("utf-8")
        .arg("--print")
        .arg("filename")
        .arg("https://www.youtube.com/watch?v=-TVw_ndGyW4")
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        println!("stderr: {}", stderr);
    }
    println!("\"{}\" len={}", stdout, stdout.len());
    println!("\"{}\" == \"{}\" ? {}", ch, stdout, ch == stdout);
    println!("");
    // "solivagant ｜ a breakcore mix [-TVw_ndGyW4].webm " len=50
    // "｜" == "solivagant ｜ a breakcore mix [-TVw_ndGyW4].webm " ? false
}
