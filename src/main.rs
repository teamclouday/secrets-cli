use std::io::Read;

use clap::Parser;
use magic_crypt::{MagicCryptTrait, new_magic_crypt};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Secrets CLI",
    long_about = "CLI tool to encode/decode secrets files"
)]
struct Args {
    /// Location of the secrets file
    #[arg(short, long, default_value = ".env")]
    filepath: String,

    /// Encoded secrets text
    #[arg(short, long)]
    text: Option<String>,

    /// Password for encoding/decoding
    #[arg(short, long, default_value = "secret")]
    password: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mcrypt = new_magic_crypt!(args.password, 256);

    // Decoding mode
    if let Some(text) = args.text {
        // Decode the base64 string to binary
        let decrypted_data = mcrypt
            .decrypt_base64_to_bytes(text)
            .expect("Failed to decrypt base64 data");

        // Write the binary data to a file
        std::fs::write(args.filepath.clone(), &decrypted_data)
            .expect(format!("Failed to write decrypted data to file: {}", args.filepath).as_str());
        println!("Decoded to {}", args.filepath);
    }
    // Encoding mode
    else {
        // Load the file contents
        let mut file = std::fs::File::open(&args.filepath)
            .expect(format!("Failed to open file: {}", args.filepath).as_str());
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect(format!("Failed to read file: {}", args.filepath).as_str());

        // Encode JSON string to base64
        let encoded_json = mcrypt.encrypt_bytes_to_base64(&buffer);
        println!("{}", encoded_json);
    }

    Ok(())
}
