use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use clap::{Parser, Subcommand};
use anyhow::{Context, Result};
use rayon::prelude::*;
use ignore::{WalkBuilder, overrides::OverrideBuilder};
use std::time::Instant;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Use blacklist (.blacklist file)
    #[arg(long)]
    blacklist: bool,

    /// Use whitelist (.whitelist file)
    #[arg(long)]
    whitelist: bool,

    /// Use .gitignore file as part of blacklist
    #[arg(long)]
    gitignore: bool,

    /// Custom blacklist patterns (comma separated)
    #[arg(long, value_delimiter = ',')]
    blacklist_patterns: Vec<String>,

    /// Custom whitelist patterns (comma separated)
    #[arg(long, value_delimiter = ',')]
    whitelist_patterns: Vec<String>,

    /// Custom blacklist file path
    #[arg(long)]
    blacklist_file: Option<String>,

    /// Custom whitelist file path
    #[arg(long)]
    whitelist_file: Option<String>,

    /// Output file path
    #[arg(short, long, default_value = "project_contents.txt")]
    output: String,

    /// Display detailed statistics about execution (files, lines, tokens)
    #[arg(short, long)]
    stats: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Show the location of configuration files
    ShowLocations,

    /// Initialize config files in home directory
    Init,
    
    /// Display version information
    Version,
    
    /// Show detailed help information
    FullHelp,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::ShowLocations) => {
            let local_blacklist_path = get_local_config_path(".blacklist");
            let local_whitelist_path = get_local_config_path(".whitelist");
            let global_blacklist_path = get_global_config_path(".contextify-blacklist");
            let global_whitelist_path = get_global_config_path(".contextify-whitelist");
            
            println!("Local blacklist file is located at: {}", local_blacklist_path.display());
            println!("Local whitelist file is located at: {}", local_whitelist_path.display());
            println!("Global blacklist file is located at: {}", global_blacklist_path.display());
            println!("Global whitelist file is located at: {}", global_whitelist_path.display());
            return Ok(());
        }
        Some(Commands::Init) => {
            init_global_config_files()?;
            println!("Global configuration files have been initialized.");
            return Ok(());
        }
        Some(Commands::Version) => {
            println!("{} v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            println!("Author: {}", env!("CARGO_PKG_AUTHORS"));
            println!("Repository: {}", env!("CARGO_PKG_REPOSITORY"));
            println!("Description: {}", env!("CARGO_PKG_DESCRIPTION"));
            return Ok(());
        }
        Some(Commands::FullHelp) => {
            println!("Contextify - {}", env!("CARGO_PKG_DESCRIPTION"));
            println!("\nDETAILED USAGE:");
            println!("  contextify [FLAGS] [OPTIONS] [COMMAND]");
            println!("\nCOMMANDS:");
            println!("  show-locations   Show the location of configuration files");
            println!("  init             Initialize config files in home directory");
            println!("  version          Display version information");
            println!("  help             Show this detailed help information");
            println!("\nFLAGS:");
            println!("  --blacklist      Use blacklist (.blacklist file)");
            println!("  --whitelist      Use whitelist (.whitelist file)");
            println!("  --gitignore      Use .gitignore file as part of blacklist");
            println!("  -s, --stats      Display detailed statistics about execution");
            println!("  -h, --help       Print help (see more with 'help')");
            println!("  -V, --version    Print version (see more with 'version')");
            println!("\nOPTIONS:");
            println!("  --blacklist-patterns <PATTERNS>    Custom blacklist patterns (comma separated)");
            println!("  --whitelist-patterns <PATTERNS>    Custom whitelist patterns (comma separated)");
            println!("  --blacklist-file <FILE>           Custom blacklist file path");
            println!("  --whitelist-file <FILE>           Custom whitelist file path");
            println!("  -o, --output <FILE>               Output file path (default: project_contents.txt)");
            println!("\nEXAMPLES:");
            println!("  contextify                        # Process all files in current directory");
            println!("  contextify --blacklist            # Use blacklist to exclude files");
            println!("  contextify --whitelist            # Use whitelist to include only specific files");
            println!("  contextify --gitignore            # Use .gitignore for exclusions");
            println!("  contextify --blacklist-patterns \"target/,.git/\" --output result.txt");
            println!("  contextify --whitelist-patterns \"*.rs,*.md\" --stats");
            return Ok(());
        }
        None => {
            // Start timing
            let start_time = Instant::now();
            
            // Get blacklist patterns
            let mut blacklist_patterns = vec![];
            
            // From command line arguments
            if !cli.blacklist_patterns.is_empty() {
                blacklist_patterns.extend(cli.blacklist_patterns.clone());
            }
            
            // From .gitignore if specified
            if cli.gitignore {
                let gitignore_patterns = read_gitignore_file()?;
                blacklist_patterns.extend(gitignore_patterns);
            }
            
            // From file
            if cli.blacklist || cli.blacklist_file.is_some() {
                let file_path = match &cli.blacklist_file {
                    Some(path) => PathBuf::from(path),
                    None => {
                        // Try local config first, then global
                        let local_path = get_local_config_path(".blacklist");
                        if local_path.exists() {
                            local_path
                        } else {
                            get_global_config_path(".contextify-blacklist")
                        }
                    }
                };
                
                let file_patterns = read_list_file(&file_path)?;
                blacklist_patterns.extend(file_patterns);
            }
            
            // Get whitelist patterns
            let mut whitelist_patterns = vec![];
            
            // From command line arguments
            if !cli.whitelist_patterns.is_empty() {
                whitelist_patterns.extend(cli.whitelist_patterns.clone());
            }
            
            // From file
            if cli.whitelist || cli.whitelist_file.is_some() {
                let file_path = match &cli.whitelist_file {
                    Some(path) => PathBuf::from(path),
                    None => {
                        // Try local config first, then global
                        let local_path = get_local_config_path(".whitelist");
                        if local_path.exists() {
                            local_path
                        } else {
                            get_global_config_path(".contextify-whitelist")
                        }
                    }
                };
                
                let file_patterns = read_list_file(&file_path)?;
                whitelist_patterns.extend(file_patterns);
            }

            // Process the project
            let stats = save_project_structure_and_files(".", &cli.output, &blacklist_patterns, &whitelist_patterns)?;
            
            // End timing
            let elapsed = start_time.elapsed();
            
            println!("Project structure and contents saved to {}", cli.output);
            
            // Display statistics if requested
            if cli.stats {
                println!("\nSTATISTICS:");
                println!("  Execution time: {:.2?}", elapsed);
                println!("  Files processed: {}", stats.file_count);
                println!("  Total lines: {}", stats.line_count);
                println!("  Total characters: {}", stats.char_count);
                println!("  Estimated tokens: {} (approx. {:.2} tokens per char)", 
                         stats.estimated_tokens,
                         if stats.char_count > 0 { stats.estimated_tokens as f64 / stats.char_count as f64 } else { 0.0 });
            }
        }
    }

    Ok(())
}

/// Statistics about processed files
struct ProcessingStats {
    file_count: usize,
    line_count: usize,
    char_count: usize,
    estimated_tokens: usize,
}

/// Get the path to a local configuration file in the current project
fn get_local_config_path(filename: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("config");
    path.push(filename);
    path
}

/// Get the path to a global configuration file in the user's home directory
fn get_global_config_path(filename: &str) -> PathBuf {
    let mut path = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push(filename);
    path
}

/// Initialize the global configuration files in the user's home directory
fn init_global_config_files() -> Result<()> {
    let blacklist_path = get_global_config_path(".contextify-blacklist");
    let whitelist_path = get_global_config_path(".contextify-whitelist");
    
    // Copy from local config if exists, or create with defaults
    if !blacklist_path.exists() {
        let local_blacklist = get_local_config_path(".blacklist");
        let mut content = if local_blacklist.exists() {
            fs::read_to_string(local_blacklist)?
        } else {
            String::from(
                ".DS_Store\n\
                 Dockerfile\n\
                 db.sqlite3\n\
                 docker-compose.yml\n\
                 project_contents.txt\n\
                 requirements.txt\n\
                 __init__.py\n\
                 .devcontainer/\n\
                 __pycache__/\n\
                 vendors/\n\
                 target/\n\
                 Cargo.lock\n\
                 .git/\n"
            )
        };
        
        // Add content from .gitignore if it exists
        let gitignore_path = Path::new(".gitignore");
        if gitignore_path.exists() {
            if let Ok(gitignore_content) = fs::read_to_string(gitignore_path) {
                // Add a header for gitignore content
                content.push_str("\n# From .gitignore\n");
                content.push_str(&gitignore_content);
            }
        }
        
        fs::write(&blacklist_path, content)?;
    }
    
    if !whitelist_path.exists() {
        let local_whitelist = get_local_config_path(".whitelist");
        let content = if local_whitelist.exists() {
            fs::read_to_string(local_whitelist)?
        } else {
            String::from(
                "*.rs\n\
                 *.md\n\
                 *.toml\n"
            )
        };
        
        fs::write(&whitelist_path, content)?;
    }
    
    Ok(())
}

/// Read a list file (.blacklist or .whitelist) and return the list of patterns
fn read_list_file(file_path: &Path) -> Result<Vec<String>> {
    match fs::read_to_string(file_path) {
        Ok(content) => Ok(content
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.trim().to_string())
            .collect()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("Warning: List file not found: {}", file_path.display());
            Ok(vec![])
        }
        Err(e) => Err(e).context(format!("Failed to read list file: {}", file_path.display())),
    }
}

/// Read the .gitignore file and return the list of patterns
fn read_gitignore_file() -> Result<Vec<String>> {
    let gitignore_path = Path::new(".gitignore");
    
    match fs::read_to_string(gitignore_path) {
        Ok(content) => {
            // Filter out comments and empty lines
            let patterns = content
                .lines()
                .filter(|line| {
                    let trimmed = line.trim();
                    !trimmed.is_empty() && !trimmed.starts_with('#')
                })
                .map(|line| line.trim().to_string())
                .collect::<Vec<String>>();
            
            Ok(patterns)
        },
        Err(e) if e.kind() == io::ErrorKind::NotFound => {
            eprintln!("Warning: .gitignore file not found");
            Ok(vec![])
        },
        Err(e) => Err(e).context("Failed to read .gitignore file"),
    }
}

/// Save the project structure and contents of all files to a text file
fn save_project_structure_and_files(
    root_path: &str,
    output_file: &str,
    blacklist_patterns: &[String],
    whitelist_patterns: &[String],
) -> Result<ProcessingStats> {
    let root_path = Path::new(root_path);
    let output_path = PathBuf::from(output_file);
    let mut project_structure = Vec::new();
    let mut file_contents = Vec::new();
    
    // Statistics
    let mut stats = ProcessingStats {
        file_count: 0,
        line_count: 0,
        char_count: 0,
        estimated_tokens: 0,
    };

    // Configure the walker with ignore patterns (.gitignore style)
    let mut builder = WalkBuilder::new(root_path);
    
    // Always skip the output file
    let output_path_clone = output_path.clone();
    builder.filter_entry(move |entry| {
        let path = entry.path();
        path != output_path_clone
    });
    
    // Add blacklist patterns
    if !blacklist_patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(root_path);
        
        // Convert each pattern to an ignore pattern
        for pattern in blacklist_patterns {
            // Prepend ! to negate the pattern (in ignore crate, ! means include, not exclude)
            let ignore_pattern = format!("!{}", pattern);
            override_builder.add(&ignore_pattern)?;
        }
        
        let overrides = override_builder.build()?;
        builder.overrides(overrides);
    }
    
    // Add whitelist patterns
    if !whitelist_patterns.is_empty() {
        let mut override_builder = OverrideBuilder::new(root_path);
        
        // First, exclude everything
        override_builder.add("!*")?;
        
        // Then include only the whitelist patterns
        for pattern in whitelist_patterns {
            override_builder.add(pattern)?;
        }
        
        let overrides = override_builder.build()?;
        builder.overrides(overrides);
    }
    
    // Skip hidden files and directories that don't match our patterns
    builder.hidden(false);
    
    // Get all the files
    let walker = builder.build();
    
    // Collect files from the walker
    let entries: Vec<_> = walker
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .collect();
    
    stats.file_count = entries.len();

    // Process files in parallel
    let results: Vec<_> = entries
        .par_iter()
        .map(|entry| {
            let path = entry.path();
            let relative_path = path.strip_prefix(root_path).unwrap_or(path);
            let path_str = relative_path.to_string_lossy().replace('\\', "/");
            
            // Capture file content
            let content = match fs::read_to_string(path) {
                Ok(content) => content,
                Err(e) => format!("Error reading file: {}", e),
            };
            
            (path_str.to_string(), content)
        })
        .collect();

    // Sort results for consistent output
    let mut sorted_results = results;
    sorted_results.sort_by(|(a, _), (b, _)| a.cmp(b));

    // Prepare output
    for (path, _) in &sorted_results {
        project_structure.push(path.clone());
    }

    for (path, content) in sorted_results {
        file_contents.push(format!("{}:\n```\n{}\n```\n", path, content));
        
        // Update statistics
        stats.line_count += content.lines().count();
        stats.char_count += content.chars().count();
        // Rough estimate: on average 1 token is about 4 characters
        stats.estimated_tokens += content.chars().count() / 4;
    }

    // Write to output file
    let mut file = File::create(output_file).context("Failed to create output file")?;
    writeln!(file, "Project Structure:")?;
    writeln!(file, "{}", project_structure.join("\n"))?;
    writeln!(file, "\nFile Contents:")?;
    write!(file, "{}", file_contents.join("\n"))?;

    Ok(stats)
}
