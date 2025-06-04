pub mod cli;
pub mod request;
pub mod environment;
pub mod response;
pub mod utils;

use anyhow::Result;
use cli::{Cli, Commands};
use environment::EnvironmentResolver;
use request::{RequestExecutor, RequestParser, RequestValidator};

pub async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Run { path, env, output, verbose } => {
            println!("🚀 Running request from: {}", path);
            let mut env_resolver = EnvironmentResolver::default();
            if let Some(env) = env {
                println!("🌍 Using environment: {}", env);
                let results = RequestParser::parse_file(&path).unwrap();
                env_resolver.load_environment_file(env.as_str()).unwrap();
                let request_handler = RequestExecutor::new();
                let _ = request_handler.execute(&results, &env_resolver).await;
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
