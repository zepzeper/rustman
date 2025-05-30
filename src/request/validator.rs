use std::path::{Path, PathBuf};
use colored::*;

use super::{RequestDefinition, RequestParser, ValidationError};

pub struct ValidationResult {
    pub file_path: PathBuf,
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>
}

impl ValidationResult {
    pub fn new(file_path: PathBuf) -> Self {
        Self {
            file_path,
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new()
        } 
    } 

    pub fn add_error(&mut self, error: ValidationError) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}


pub struct RequestValidator;

impl RequestValidator {
    pub fn validate_file<P: AsRef<Path>>(path: P) -> ValidationResult {
        let path = path.as_ref().to_path_buf();
        let mut result = ValidationResult::new(path.clone());

        match RequestParser::parse_file(&path) {
            Ok(request) => {
                if let Err(error) = request.validate() {
                    result.add_error(error)
                }

                Self::check_warning(&request, &mut result);
            }
            Err(error) => {
                result.add_error(ValidationError::InvalidJson(error.to_string()));
            }
        }

        result
    }

    pub fn validate_directory<P: AsRef<Path>>(dir: P) -> Vec<ValidationResult> {
        let parse_results = RequestParser::parse_directory(dir);
        let mut validation_results = Vec::new();


        for (path, parse_result) in parse_results {
            let mut result = ValidationResult::new(path.clone());

            match parse_result {
                Ok(request) => {
                    if let Err(error) = request.validate() {
                        result.add_error(error);
                    }

                    Self::check_warning(&request, &mut result);
                }
                Err(error) => {
                    result.add_error(ValidationError::InvalidJson(error.to_string()));
                }
            }

            validation_results.push(result);
        }

        validation_results
    }

    fn check_warning(request: &RequestDefinition, result: &mut ValidationResult) {
        // Check for hardcoded URLs (should use templates)
        if !request.url.contains("{{") && 
        (request.url.starts_with("http://") || request.url.starts_with("https://")) {
            result.add_warning("Consider using environment variables for URLs".to_string());
        }

        // Check for hardcoded auth tokens
        if let Some(auth) = &request.auth {
            match auth {
                crate::request::AuthConfig::Bearer { token } => {
                    if !token.contains("{{") {
                        result.add_warning("Consider using environment variables for auth tokens".to_string());
                    }
                }
                crate::request::AuthConfig::ApiKey { value, .. } => {
                    if !value.contains("{{") {
                        result.add_warning("Consider using environment variables for API keys".to_string());
                    }
                }
                _ => {}
            }
        }

        // Check for hardcoded URLs (should use templates)
        if !request.url.contains("{{") && 
        (request.url.starts_with("http://") || request.url.starts_with("https://")) {
            result.add_warning("Consider using environment variables for URLs".to_string());
        }

    }

    pub fn print_validation_results(results: &[ValidationResult]) {
        let mut total_files = 0;
        let mut valid_files = 0;
        let mut total_errors = 0;
        let mut total_warnings = 0;

        println!("{}", "🔍 Validation Results".bold().cyan());
        println!();

        for result in results {
            total_files += 1;
            total_errors += result.errors.len();
            total_warnings += result.warnings.len();

            let file_name = result.file_path.file_name()
                .unwrap_or_default()
                .to_string_lossy();

            if result.is_valid {
                valid_files += 1;
                if result.warnings.is_empty() {
                    println!("  {} {}", "✅".green(), file_name.green());
                } else {
                    println!("  {} {} {}", "⚠️".yellow(), file_name.yellow(), 
                        format!("({} warnings)", result.warnings.len()).dimmed());
                    for warning in &result.warnings {
                        println!("     {} {}", "⚠️".yellow(), warning.yellow());
                    }
                }
            } else {
                println!("  {} {} {}", "❌".red(), file_name.red(),
                    format!("({} errors)", result.errors.len()).dimmed());
                for error in &result.errors {
                    println!("     {} {}", "❌".red(), error.to_string().red());
                }
                for warning in &result.warnings {
                    println!("     {} {}", "⚠️".yellow(), warning.yellow());
                }
            }
        }

        println!();
        println!("{}", "📊 Summary".bold().cyan());
        println!("  Files processed: {}", total_files);
        println!("  Valid files: {} {}", 
            valid_files.to_string().green(),
            if valid_files == total_files { "🎉" } else { "" });
        println!("  Invalid files: {}", (total_files - valid_files).to_string().red());
        println!("  Total errors: {}", total_errors.to_string().red());
        println!("  Total warnings: {}", total_warnings.to_string().yellow());

        if valid_files == total_files && total_warnings == 0 {
            println!("\n{}", "🎉 All files are valid with no warnings!".green().bold());
        } else if valid_files == total_files {
            println!("\n{}", "✅ All files are valid (with some warnings)".green());
        }
    }
}
