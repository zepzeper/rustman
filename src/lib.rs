pub mod cli;
pub mod request;
pub mod environment;
pub mod response;
pub mod utils;

use anyhow::{Context, Result};
use cli::{Cli, Commands};
use environment::EnvironmentResolver;
use request::{RequestDefinition, RequestExecutor, RequestParser, RequestValidator};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Run { path, env, output, verbose } => {
            if verbose {
                println!("🚀 Running request from: {}", path);
            }

            let mut env_resolver = EnvironmentResolver::default();
            if let Some(env_file_path) = env {
                if verbose {
                    println!("🌍 Loading environment from: {}", env_file_path);
                }
                env_resolver
                    .load_environment_file(env_file_path.as_str())
                    .with_context(|| format!("Failed to load environment file: {}", env_file_path))?;
                if verbose {
                    if let Some(name) = env_resolver.active_environment_name() {
                        println!("  -> Environment '{}' loaded.", name);
                    } else {
                        println!("  -> Environment loaded (name not available).");
                    }
                }
            } else if verbose {
                println!("🌍 No environment file specified. Using default (empty) environment.");
            }

            if verbose {
                println!("📄 Parsing request file: {}", path);
            }
            let raw_request_def: RequestDefinition = RequestParser::parse_file(&path)
                .with_context(|| format!("Failed to parse request file: {}", path))?;

            if verbose {
                println!("🔧 Resolving request definition with environment variables...");
            }
            let resolved_request_def = raw_request_def.resolve_with_env(&env_resolver);
            if verbose {
                println!("  -> Resolved Request: {:#?}", resolved_request_def);
            }

            let request_executor = RequestExecutor::new();
            if verbose {
                println!("⏳ Executing request: {}...", resolved_request_def.unwrap().name);
            }


            Ok(())
        }
        Commands::Validate { path } => {
            let path_obj = std::path::Path::new(&path);

            if path_obj.is_file() {
                let results = RequestValidator::validate_file(path_obj);
                RequestValidator::print_validation_results(&[results]);
            } else if path_obj.is_dir() {
                let results = RequestValidator::validate_directory(path_obj);
                RequestValidator::print_validation_results(&results);
            } else {
                eprintln!("Error: Path '{}' is not a valid file or directory.", path);
            }

            Ok(())
        }
        Commands::List { directory } => {
            println!("📋 Listing requests in: {}", directory);
            let files = utils::find_request_files(&directory)
                .with_context(|| format!("Failed to find request files in directory: {}", directory))?;

            if files.is_empty() {
                println!("  -> No request files found in '{}'.", directory);
            } else {
                for file in files {
                    // .display() is cleaner for printing paths
                    println!("    📄 {}", file.display());
                }
            }

            Ok(())
        }
    }
}
