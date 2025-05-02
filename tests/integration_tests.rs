use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::env;
use std::io;

/// Helper function to get the path to the compiled binary
fn get_binary_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if cfg!(debug_assertions) {
        path.push("target/debug/contextify");
    } else {
        path.push("target/release/contextify");
    }
    
    // Добавляем .exe для Windows
    if cfg!(windows) {
        path.set_extension("exe");
    }
    
    path
}

/// Create test directory with proper path
fn get_test_dir(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests");
    path.push("data");
    path.push(name);
    
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("Failed to create parent directory");
    }
    
    path
}

/// Helper function to create a test directory with sample files
fn setup_test_directory(test_dir: &Path) -> std::io::Result<()> {
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    
    fs::create_dir_all(test_dir)?;
    
    // Create sample files with different extensions
    fs::write(test_dir.join("file1.rs"), "fn main() {\n    println!(\"Hello, world!\");\n}")?;
    fs::write(test_dir.join("file2.md"), "# Title\n\nThis is a markdown file.")?;
    fs::write(test_dir.join("file3.txt"), "Plain text file.")?;
    fs::write(test_dir.join("file4.json"), "{\"key\": \"value\"}")?;
    
    // Create subdirectory with files
    let subdir = test_dir.join("subdir");
    fs::create_dir(&subdir)?;
    fs::write(subdir.join("subfile1.rs"), "struct Test {\n    field: i32\n}")?;
    fs::write(subdir.join("subfile2.txt"), "Another text file.")?;
    
    // Create .gitignore
    fs::write(test_dir.join(".gitignore"), "*.json\n*.txt\n")?;
    
    // Create config files
    let config_dir = test_dir.join("config");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join(".blacklist"), "*.json\n")?;
    fs::write(config_dir.join(".whitelist"), "*.rs\n*.md\n")?;
    
    Ok(())
}

/// Clean up test directory after tests
fn cleanup_test_directory(test_dir: &Path) -> std::io::Result<()> {
    if test_dir.exists() {
        fs::remove_dir_all(test_dir)?;
    }
    Ok(())
}

/// Helper function to check the output content
fn check_output_content(output_content: &str, expected_files: &[&str], excluded_files: &[&str]) {
    // Debug output - check if specific files are present
    for &file in expected_files {
        if output_content.contains(file) {
            println!("✓ Expected file {} is PRESENT in output", file);
        } else {
            println!("✗ Expected file {} is MISSING from output", file);
        }
        assert!(output_content.contains(file), "Expected file {} is missing from output", file);
    }
    
    for &file in excluded_files {
        if !output_content.contains(file) {
            println!("✓ Excluded file {} is correctly NOT present in output", file);
        } else {
            println!("✗ Excluded file {} is incorrectly present in output", file);
        }
        assert!(!output_content.contains(file), "Excluded file {} is incorrectly present in output", file);
    }
}

/// Test basic functionality without any flags
#[test]
fn test_default_run() -> io::Result<()> {
    let test_dir = get_test_dir("default_test");
    setup_test_directory(&test_dir)?;
    
    let output_file = test_dir.join("output.txt");
    
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .current_dir(&test_dir)
        .arg("--output")
        .arg(output_file.file_name().unwrap())
        .output()?;
    
    println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    // By default, .gitignore should be respected
    let output_content = fs::read_to_string(&output_file)?;
    
    let expected_files = ["file1.rs", "file2.md", "subfile1.rs"];
    let excluded_files = ["file3.txt", "file4.json", "subfile2.txt"];
    
    check_output_content(&output_content, &expected_files, &excluded_files);
    
    cleanup_test_directory(&test_dir)?;
    Ok(())
}

/// Test explicit --gitignore flag
#[test]
fn test_gitignore_flag() -> io::Result<()> {
    let test_dir = get_test_dir("gitignore_test");
    setup_test_directory(&test_dir)?;
    
    let output_file = test_dir.join("output.txt");
    
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .current_dir(&test_dir)
        .arg("--gitignore")
        .arg("--output")
        .arg(output_file.file_name().unwrap())
        .output()?;
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    // With --gitignore, .gitignore patterns should be applied
    let output_content = fs::read_to_string(&output_file)?;
    
    let expected_files = ["file1.rs", "file2.md", "subfile1.rs"];
    let excluded_files = ["file3.txt", "file4.json", "subfile2.txt"];
    
    check_output_content(&output_content, &expected_files, &excluded_files);
    
    cleanup_test_directory(&test_dir)?;
    Ok(())
}

/// Test no-gitignore flag to disable gitignore processing
#[test]
fn test_no_gitignore_flag() -> io::Result<()> {
    let test_dir = get_test_dir("no_gitignore_test");
    setup_test_directory(&test_dir)?;
    
    let output_file = test_dir.join("output.txt");
    
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .current_dir(&test_dir)
        .arg("--no-gitignore")
        .arg("--output")
        .arg(output_file.file_name().unwrap())
        .output()?;
    
    println!("Command stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("Command stderr: {}", String::from_utf8_lossy(&output.stderr));
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    // With --no-gitignore, all files should be included
    let output_content = fs::read_to_string(&output_file)?;
    
    let expected_files = ["file1.rs", "file2.md", "file3.txt", "file4.json", "subfile1.rs", "subfile2.txt"];
    let excluded_files: [&str; 0] = [];
    
    check_output_content(&output_content, &expected_files, &excluded_files);
    
    cleanup_test_directory(&test_dir)?;
    Ok(())
}

/// Test blacklist functionality
#[test]
fn test_blacklist_only() -> io::Result<()> {
    let test_dir = get_test_dir("blacklist_only_test");
    setup_test_directory(&test_dir)?;
    
    let output_file = test_dir.join("output.txt");
    
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .current_dir(&test_dir)
        .arg("--blacklist")
        .arg("--no-gitignore")  // Disable gitignore to isolate blacklist functionality
        .arg("--output")
        .arg(output_file.file_name().unwrap())
        .output()?;
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    // With --blacklist, only .json files should be excluded
    let output_content = fs::read_to_string(&output_file)?;
    
    let expected_files = ["file1.rs", "file2.md", "file3.txt", "subfile1.rs", "subfile2.txt"];
    let excluded_files = ["file4.json"];
    
    check_output_content(&output_content, &expected_files, &excluded_files);
    
    cleanup_test_directory(&test_dir)?;
    Ok(())
}

/// Test whitelist functionality
#[test]
fn test_whitelist_only() -> io::Result<()> {
    let test_dir = get_test_dir("whitelist_only_test");
    setup_test_directory(&test_dir)?;
    
    let output_file = test_dir.join("output.txt");
    
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .current_dir(&test_dir)
        .arg("--whitelist")
        .arg("--no-gitignore")  // Disable gitignore to isolate whitelist functionality
        .arg("--output")
        .arg(output_file.file_name().unwrap())
        .output()?;
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    // With --whitelist, only .rs and .md files should be included
    let output_content = fs::read_to_string(&output_file)?;
    
    let expected_files = ["file1.rs", "file2.md", "subfile1.rs"];
    let excluded_files = ["file3.txt", "file4.json", "subfile2.txt"];
    
    check_output_content(&output_content, &expected_files, &excluded_files);
    
    cleanup_test_directory(&test_dir)?;
    Ok(())
}

/// Test custom patterns
#[test]
fn test_custom_patterns_only() -> io::Result<()> {
    let test_dir = get_test_dir("custom_patterns_test");
    setup_test_directory(&test_dir)?;
    
    let output_file = test_dir.join("output.txt");
    
    let binary = get_binary_path();
    
    let output = Command::new(&binary)
        .current_dir(&test_dir)
        .arg("--blacklist-patterns")
        .arg("*.txt,subdir")
        .arg("--no-gitignore")  // Disable gitignore to isolate custom patterns
        .arg("--output")
        .arg(output_file.file_name().unwrap())
        .output()?;
    
    assert!(output.status.success());
    assert!(output_file.exists());
    
    // With --blacklist-patterns, .txt files and everything in subdir should be excluded
    let output_content = fs::read_to_string(&output_file)?;
    
    let expected_files = ["file1.rs", "file2.md", "file4.json"];
    let excluded_files = ["file3.txt", "subfile1.rs", "subfile2.txt"];
    
    check_output_content(&output_content, &expected_files, &excluded_files);
    
    cleanup_test_directory(&test_dir)?;
    Ok(())
} 