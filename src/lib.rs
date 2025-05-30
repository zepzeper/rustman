pub mod cli;
pub mod request;
pub mod environment;
pub mod response;
pub mod utils;

use anyhow::Result;
use cli::{Cli, Commands};
use request::RequestValidator;

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Run { path, env, output, verbose } => {
            println!("🚀 Running request from: {}", path);
            if let Some(env) = env {
                println!("🌍 Using environment: {}", env);
            }
            if let Some(output) = output {
                println!("📄 Output file: {}", output);
            }


            Ok(())
        }
        Commands::Validate { path } => {
            let path = std::path::Path::new(&path);

            if path.is_file() {
                let results = RequestValidator::validate_file(path);
                RequestValidator::print_validation_results(&[results]);
            } else {
                let results = RequestValidator::validate_directory(path);
                RequestValidator::print_validation_results(&results);
            }


            Ok(())
        }
        Commands::List { directory } => {
            println!("📋 Listing requests in: {}", directory);
            let paths = utils::find_request_files(&directory);

            if let Ok(files) = paths {
                for file in files {
                    println!("  📄 {:?}", file);
                }
            };

            Ok(())
        }
    }
}
