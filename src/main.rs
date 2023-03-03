use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenv::dotenv;
use std::path::PathBuf;

pub mod create_story;
pub mod group_by_variant;
/// Command line tool for automating internal workflows for the Clickconcepts frontend team
#[derive(Parser)]
#[command(author, version, about)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Outputs reviews grouped by product variant from yotpo product reviews export
    GroupByVariant {
        #[arg(value_name = "REVIEWS_FILE")]
        file_path: PathBuf,
    },
    /// Create a Storybook story file from a frontastic component schema
    CreateStory {
        #[arg(value_name = "SCHEMA_FILE")]
        file_path: PathBuf,
    },
}
#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::CreateStory { file_path }) => create_story::create_story(file_path)?,
        Some(Commands::GroupByVariant { file_path }) => {
            group_by_variant::group_by_variant(file_path).await?
        }
        None => (),
    };

    Ok(())
}
