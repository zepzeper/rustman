use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "rmcli")]
#[command(about = "A CLI tool for HTTP requests")]
#[command(version = "0.1.0")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Run {
        /// Path to request file or directory
        #[arg(value_name = "FILE")]
        path: String,
        
        /// Environment to use
        #[arg(short, long)]
        env: Option<String>,
        
        /// Output file for response
        #[arg(short, long)]
        output: Option<String>,
        
        /// Verbose output
        #[arg(short, long, default_value = "false")]
        verbose: bool,
    },
    
    /// Validate request files
    Validate {
        /// Path to request file or directory
        #[arg(value_name = "FILE")]
        path: String,
    },
    
    /// List available requests
    List {
        /// Directory to search
        #[arg(value_name = "DIR", default_value = ".")]
        directory: String,
    },
}
