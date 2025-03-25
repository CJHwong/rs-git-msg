use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use std::process;

mod ai;
mod commit;
mod git;

use ai::provider_factory::create_provider;

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq)]
enum Provider {
    Ollama,
    #[value(name = "openai")]
    OpenAI,
}

impl Provider {
    fn default_model(&self) -> &'static str {
        match self {
            Provider::Ollama => "qwen2.5-coder",
            Provider::OpenAI => "gpt-4o-mini",
        }
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of commit messages to generate (1-5)
    #[arg(short = 'n', long = "number", default_value_t = 1)]
    numbers: u8,

    /// Additional context or instructions for the AI
    #[arg(short = 'i', long)]
    instructions: Option<String>,
    
    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,
    
    /// AI provider to use
    #[arg(short = 'p', long, value_enum, default_value_t = Provider::Ollama)]
    provider: Provider,
    
    /// Model name to use
    #[arg(short = 'm', long)]
    model: Option<String>,
    
    /// API key for the provider (not needed for Ollama)
    #[arg(short = 'k', long)]
    api_key: Option<String>,
    
    /// API base URL (defaults to provider's standard URL)
    #[arg(short = 'u', long)]
    api_url: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    
    let api_key = args.api_key.or_else(|| std::env::var("RS_GIT_MSG_API_KEY").ok());
    
    if args.numbers < 1 || args.numbers > 5 {
        eprintln!("Error: Number of messages must be between 1 and 5");
        process::exit(1);
    }

    // Use the model provided by the user or fall back to the provider's default
    let model = args.model.unwrap_or_else(|| args.provider.default_model().to_string());

    if args.verbose {
        println!("Opening git repository...");
    }
    
    let repo = git::Repository::open_current_dir(args.verbose)
        .context("Failed to open git repository")?;
    
    let branch_name = repo.get_branch_name()
        .context("Failed to get branch name")?;
        
    if args.verbose {
        println!("Current branch: {}", branch_name);
        println!("Reading staged changes...");
    }
    
    let diff = repo.get_staged_diff()
        .context("Failed to get staged diff")?;
    
    if diff.is_empty() {
        println!("No staged changes found. Stage some changes first with 'git add'");
        println!("Make sure you have staged changes using 'git add <file>' before running this command");
        process::exit(1);
    }

    if args.verbose {
        println!("Found staged changes, generating commit message...");
        println!("Using provider: {:?} with model: {}", args.provider, model);
    }
    
    let ai_provider = create_provider(
        args.provider,
        &model,
        api_key.as_deref(),
        args.api_url.as_deref(),
        args.verbose,
    )?;
    
    let generator = commit::CommitMessageGenerator::new(ai_provider);
    
    if args.verbose {
        println!("Generating commit message(s)...");
    }
    
    let messages = generator.generate(
        &diff,
        &branch_name,
        args.numbers,
        args.instructions.as_deref(),
    ).await.context("Failed to generate commit message")?;
    
    for message in &messages {
        println!("{}", message);
    }

    Ok(())
}
