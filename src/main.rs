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
    #[arg(short, long)]
    filepath: String,

    /// Encoded secrets text
    #[arg(short, long)]
    text: Option<String>,

    /// Password for encoding/decoding
    #[arg(short, long, default_value = "secret")]
    password: String,

    /// Whether to copy the output to clipboard
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    copy: bool,

    /// Whether to overwrite the file if it exists
    #[arg(short, long, default_value_t = false, action = clap::ArgAction::SetTrue)]
    overwrite: bool,
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

        // If the file already exists, make a backup
        if std::path::Path::new(&args.filepath).exists() {
            if args.overwrite {
                println!("Overwriting existing {}", args.filepath);
            } else {
                let backup_path = format!("{}.bak", args.filepath);
                std::fs::rename(&args.filepath, &backup_path)
                    .expect(format!("Failed to rename existing file to {}", backup_path).as_str());
                println!("Created backup {}", backup_path);
            }
        }

        // Write the binary data to a file
        std::fs::write(&args.filepath, &decrypted_data)
            .expect(format!("Failed to write decrypted data to {}", args.filepath).as_str());
        println!("Decoded to {}", args.filepath);
    }
    // Encoding mode
    else {
        // Load the file contents
        let mut file = std::fs::File::open(&args.filepath)
            .expect(format!("Failed to open {}", args.filepath).as_str());
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)
            .expect(format!("Failed to read {}", args.filepath).as_str());

        // Encode JSON string to base64
        let encoded_json = mcrypt.encrypt_bytes_to_base64(&buffer);

        // Print the encoded string
        println!("{}", encoded_json);

        if args.copy {
            // Copy the encoded string to clipboard
            let mut clipboard = arboard::Clipboard::new().expect("Failed to open clipboard");
            clipboard
                .set_text(encoded_json.clone())
                .expect("Failed to copy to clipboard");
            println!("Copied to clipboard");
        }
    }

    Ok(())
}
