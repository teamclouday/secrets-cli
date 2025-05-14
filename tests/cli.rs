use assert_cmd::Command;
use std::io::Write;

#[test]
fn test_encode_decode() {
    let input_filename = "testfile1.env";
    let output_filename = "testfile2.env";

    // Create a temporary file for the test
    let test_content = "test secret content";
    write!(
        &mut std::fs::File::create(input_filename).unwrap(),
        "test secret content"
    )
    .unwrap();

    // Run the encode command
    let mut encode_cmd = Command::cargo_bin("secrets-cli").unwrap();
    let output = encode_cmd
        .arg("--filepath")
        .arg(input_filename)
        .arg("--password")
        .arg("testpassword")
        .output()
        .expect("Failed to execute encode command");

    assert!(output.status.success(), "Encode command failed");

    // Get the encoded text from stdout
    let encoded_text = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert!(!encoded_text.is_empty(), "Encoded text should not be empty");

    // Step 2: Run the decode command with the encoded text
    let mut decode_cmd = Command::cargo_bin("secrets-cli").unwrap();
    let decode_assert = decode_cmd
        .arg("--filepath")
        .arg(output_filename)
        .arg("--text")
        .arg(encoded_text)
        .arg("--password")
        .arg("testpassword")
        .assert();

    // Verify decode command succeeded
    decode_assert.success();

    // Verify the decoded content matches the original
    let decoded_content = std::fs::read_to_string(&output_filename).unwrap();
    assert_eq!(
        decoded_content, test_content,
        "Decoded content doesn't match original"
    );

    // Clean up the temparary files
    std::fs::remove_file(&input_filename).unwrap_or(());
    std::fs::remove_file(&output_filename).unwrap_or(());
}
