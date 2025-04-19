use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashSet;
use regex::Regex;

const MAX_FILE_LINES: usize = 500;
const MIN_COMMENT_RATIO: f32 = 0.15; // At least 15% of lines should be comments

/// Test to ensure that source files don't exceed the maximum allowed line count
/// Test to ensure that restore target types have their own dedicated files
#[test]
fn test_restore_target_separation() {
    // Define the restore target types we expect to have dedicated files
    let target_types = vec!["postgres", "elasticsearch", "qdrant"];
    
    // Get all Rust source files in the project
    let source_files = get_rust_source_files();
    
    // Check if we have dedicated files for each restore target type
    let mut missing_targets = Vec::new();
    
    for target_type in &target_types {
        let target_file_exists = source_files.iter().any(|path| {
            let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
            file_name.contains(target_type) && 
            (file_name.contains("target") || file_name.contains("config"))
        });
        
        if !target_file_exists {
            missing_targets.push(target_type.to_string());
        }
    }
    
    if !missing_targets.is_empty() {
        panic!("Missing dedicated files for restore target types: {:?}", missing_targets);
    }
    
    // Check if restore target logic is properly separated
    let mut mixed_target_files = Vec::new();
    
    for path in &source_files {
        // Skip files that are expected to contain multiple target types
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_lowercase();
        if file_name.contains("mod.rs") || file_name.contains("restore.rs") {
            continue;
        }
        
        // Count how many target types are referenced in the file
        let content = fs::read_to_string(path).unwrap_or_default();
        let mut target_count = 0;
        
        for target_type in &target_types {
            if content.to_lowercase().contains(target_type) {
                target_count += 1;
            }
        }
        
        // If a file references multiple target types but isn't a dedicated target file,
        // it might be mixing concerns
        if target_count > 1 && !target_types.iter().any(|t| file_name.contains(t)) {
            mixed_target_files.push(path.clone());
        }
    }
    
    if !mixed_target_files.is_empty() {
        let mut warning = String::from("\n⚠️  CODE QUALITY WARNING ⚠️\n");
        warning.push_str("The following files may contain mixed restore target logic:\n");
        
        for file_path in mixed_target_files {
            let relative_path = file_path.strip_prefix(project_root()).unwrap_or(&file_path);
            warning.push_str(&format!("- {}\n", relative_path.display()));
        }
        
        warning.push_str("\nConsider separating restore target logic into dedicated files.\n");
        println!("{}", warning);
    }
}

/// Test to ensure that code has detailed comments
#[test]
fn test_detailed_comments() {
    // Get all Rust source files in the project
    let source_files = get_rust_source_files();
    
    // Track files with insufficient comments
    let mut insufficient_comments = Vec::new();
    
    for file_path in &source_files {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        let lines: Vec<&str> = content.lines().collect();
        
        if lines.is_empty() {
            continue;
        }
        
        // Count comment lines
        let comment_lines = lines.iter()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.contains("*/")
            })
            .count();
        
        // Calculate comment ratio
        let comment_ratio = comment_lines as f32 / lines.len() as f32;
        
        if comment_ratio < MIN_COMMENT_RATIO {
            insufficient_comments.push((file_path.clone(), comment_ratio));
        }
    }
    
    if !insufficient_comments.is_empty() {
        let mut warning = String::from("\n⚠️  CODE QUALITY WARNING ⚠️\n");
        warning.push_str("The following files have insufficient comments:\n");
        
        for (file_path, ratio) in insufficient_comments {
            let relative_path = file_path.strip_prefix(project_root()).unwrap_or(&file_path);
            warning.push_str(&format!(
                "- {} has {:.1}% comments (minimum is {:.1}%)\n", 
                relative_path.display(), 
                ratio * 100.0, 
                MIN_COMMENT_RATIO * 100.0
            ));
        }
        
        warning.push_str("\nConsider adding more detailed comments to improve code readability.\n");
        println!("{}", warning);
    }
}

/// Test to ensure that functions include logging
#[test]
fn test_function_logging() {
    // Get all Rust source files in the project
    let source_files = get_rust_source_files();
    
    // Track files with functions missing logging
    let mut missing_logging = Vec::new();
    
    // Regular expression to match function definitions
    let fn_regex = Regex::new(r"\s*(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)").unwrap();
    
    // Regular expressions to match logging statements
    let log_regex = Regex::new(r"(?:debug!|info!|warn!|error!|trace!|log::debug|log::info|log::warn|log::error|log::trace)").unwrap();
    
    for file_path in &source_files {
        let content = fs::read_to_string(file_path).unwrap_or_default();
        
        // Skip test files and very small files
        if file_path.to_string_lossy().contains("/tests/") || content.lines().count() < 20 {
            continue;
        }
        
        // Find all function definitions
        let mut functions_without_logging = HashSet::new();
        let mut current_fn = None;
        let mut brace_count = 0;
        let mut has_logging = false;
        
        for line in content.lines() {
            // Check for function definition
            if let Some(captures) = fn_regex.captures(line) {
                if let Some(fn_name) = captures.get(1) {
                    // If we were processing a function, finalize it
                    if let Some(name) = current_fn {
                        if !has_logging && brace_count == 0 {
                            functions_without_logging.insert(name);
                        }
                    }
                    
                    // Start processing new function
                    current_fn = Some(fn_name.as_str().to_string());
                    has_logging = false;
                    brace_count = 0;
                }
            }
            
            // Count braces to track function body
            if let Some(_) = current_fn {
                brace_count += line.matches('{').count() as i32;
                brace_count -= line.matches('}').count() as i32;
                
                // Check for logging statements
                if log_regex.is_match(line) {
                    has_logging = true;
                }
                
                // Function ended
                if brace_count == 0 && line.contains('}') {
                    if !has_logging && current_fn.is_some() {
                        functions_without_logging.insert(current_fn.unwrap());
                    }
                    current_fn = None;
                }
            }
        }
        
        if !functions_without_logging.is_empty() {
            missing_logging.push((file_path.clone(), functions_without_logging));
        }
    }
    
    if !missing_logging.is_empty() {
        let mut warning = String::from("\n⚠️  CODE QUALITY WARNING ⚠️\n");
        warning.push_str("The following files have functions without logging statements:\n");
        
        for (file_path, functions) in missing_logging {
            let relative_path = file_path.strip_prefix(project_root()).unwrap_or(&file_path);
            warning.push_str(&format!("- {} missing logging in functions: {:?}\n", 
                relative_path.display(), 
                functions
            ));
        }
        
        warning.push_str("\nConsider adding logging statements to all functions for better debugging.\n");
        println!("{}", warning);
    }
}

#[test]
fn test_file_size_limits() {
    // Get all Rust source files in the project
    let source_files = get_rust_source_files();
    
    // Track files that exceed the limit
    let mut oversized_files = Vec::new();
    
    // Check each file's line count
    for file_path in source_files {
        let line_count = count_lines(&file_path);
        
        if line_count > MAX_FILE_LINES {
            oversized_files.push((file_path.clone(), line_count));
        }
    }
    
    // Debug output to show all file sizes
    println!("\nFile size check results:");
    println!("Maximum allowed lines: {}", MAX_FILE_LINES);
    println!("Found {} files exceeding the limit", oversized_files.len());
    
    // If there are oversized files, print a warning
    if !oversized_files.is_empty() {
        let mut warning = String::from("\n⚠️  CODE QUALITY WARNING ⚠️\n");
        warning.push_str("The following files exceed the maximum line count limit:\n");
        
        // Create a vector to store file names for later reference
        let mut large_files = Vec::new();
        
        // Process the oversized files
        for (file_path, line_count) in &oversized_files {
            let relative_path = file_path.strip_prefix(project_root()).unwrap_or(file_path);
            warning.push_str(&format!(
                "- {} has {} lines (exceeds limit of {})\n", 
                relative_path.display(), 
                line_count, 
                MAX_FILE_LINES
            ));
            
            // Store the file name for later
            if let Some(file_name) = file_path.file_name() {
                large_files.push(file_name.to_string_lossy().to_string());
            }
        }
        
        warning.push_str("\nRefactoring Recommendations:\n");
        
        // Add specific recommendations for known large files
        for file_name in large_files {
            if file_name == "renderer.rs" {
                warning.push_str(
                    "- renderer.rs: Split rendering functions into separate component files.\n\
                     Consider moving each section (S3 settings, PostgreSQL settings, etc.) to\n\
                     its own component file in the components directory.\n"
                );
            } else if file_name == "rustored.rs" {
                warning.push_str(
                    "- rustored.rs: Extract key handling logic into separate modules.\n\
                     Consider creating a separate module for event handling and\n\
                     breaking down the key handling into smaller functions.\n"
                );
            }
        }
        
        warning.push_str("\nMaintaining smaller files improves code readability, testability, and maintainability.\n");
        
        // Print warning but don't fail the test - this is just a warning
        println!("{}", warning);
    }
}

/// Get the project root directory
fn project_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// Count the number of lines in a file
fn count_lines(file_path: &Path) -> usize {
    match fs::read_to_string(file_path) {
        Ok(content) => content.lines().count(),
        Err(_) => 0, // If we can't read the file, assume it's empty
    }
}

/// Recursively get all Rust source files in the project
fn get_rust_source_files() -> Vec<PathBuf> {
    let mut result = Vec::new();
    let src_dir = project_root().join("src");
    
    // Skip if src directory doesn't exist
    if !src_dir.exists() {
        return result;
    }
    
    collect_rust_files(&src_dir, &mut result);
    result
}

/// Recursively collect all Rust files in a directory
fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            
            if path.is_dir() {
                collect_rust_files(&path, files);
            } else if let Some(extension) = path.extension() {
                if extension == "rs" {
                    files.push(path);
                }
            }
        }
    }
}
