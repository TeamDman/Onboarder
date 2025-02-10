fn main() {
    // Open keyfile.
    let filename = "localhost.key";

    let contents =
        std::fs::read_to_string(filename).expect("Something went wrong reading the file");
    println!("With text:\n{}", contents);

    let keyfile = std::fs::File::open(filename)
        .map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("failed to open {}: {}", filename, e),
            )
        })
        .unwrap();
    let mut reader = std::io::BufReader::new(keyfile);

    // Load and return a single private key.
    let keys = rustls_pemfile::pkcs8_private_keys(&mut reader)
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::Other, "failed to load private key"))
        .unwrap();
    if keys.len() != 1 {
        Err::<(), std::io::Error>(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("expected a single private key, got {} instead", keys.len()),
        ))
        .unwrap();
    }

    let key = rustls::PrivateKey(keys[0].clone());
    println!("{:?}", key);
}
