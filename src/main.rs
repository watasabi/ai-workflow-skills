use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    ai_workflow_skills::cli::run(ai_workflow_skills::cli::Cli::parse())
}
