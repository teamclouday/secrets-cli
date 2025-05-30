use std::fmt;

use clap::Parser;
use console::{Style, style};
use magic_crypt::{MagicCryptTrait, new_magic_crypt};
use similar::{ChangeTag, TextDiff};

struct Line(Option<usize>);

impl fmt::Display for Line {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.0 {
            None => write!(f, "    "),
            Some(idx) => write!(f, "{:<4}", idx + 1),
        }
    }
}

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

    /// Another file to compare with the secrets file
    #[arg(long)]
    compare: Option<String>,

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

    // Compare mode
    if let Some(compare_path) = args.compare {
        // Load the secrets file
        let secrets_content = std::fs::read_to_string(&args.filepath)
            .expect(format!("Failed to read {}", args.filepath).as_str())
            .replace("\r\n", "\n");

        // Load the comparison file
        let compare_content = std::fs::read_to_string(&compare_path)
            .expect(format!("Failed to read {}", compare_path).as_str())
            .replace("\r\n", "\n");

        // Create a text diff
        let diff = TextDiff::from_lines(&secrets_content, &compare_content);
        for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
            if idx > 0 {
                println!("{:-^1$}", "-", 80);
            }
            for op in group {
                for change in diff.iter_inline_changes(op) {
                    let (sign, s) = match change.tag() {
                        ChangeTag::Delete => ("-", Style::new().red()),
                        ChangeTag::Insert => ("+", Style::new().green()),
                        ChangeTag::Equal => (" ", Style::new().dim()),
                    };
                    print!(
                        "{}{} |{}",
                        style(Line(change.old_index())).dim(),
                        style(Line(change.new_index())).dim(),
                        s.apply_to(sign).bold(),
                    );
                    for (emphasized, value) in change.iter_strings_lossy() {
                        if emphasized {
                            print!("{}", s.apply_to(value).underlined().on_black());
                        } else {
                            print!("{}", s.apply_to(value));
                        }
                    }
                    if change.missing_newline() {
                        println!();
                    }
                }
            }
        }
    }
    // Decoding mode
    else if let Some(text) = args.text {
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
        let buffer = std::fs::read(&args.filepath)
            .expect(format!("Failed to read {}", args.filepath).as_str());

        // Encode file content to base64
        let encoded = mcrypt.encrypt_bytes_to_base64(&buffer);

        // Print the encoded string
        println!("{}", encoded);

        if args.copy {
            // Copy the encoded string to clipboard
            let mut clipboard = arboard::Clipboard::new().expect("Failed to open clipboard");
            clipboard
                .set_text(encoded.clone())
                .expect("Failed to copy to clipboard");
            println!("Copied to clipboard");
        }
    }

    Ok(())
}
