mod cli;
mod ticket;
mod utils;
mod web;
mod tui;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

use ticket::TicketManager;
use cli::Cli;
use utils::find_tickets_dir;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    // Determine tickets directory
    let tickets_dir = if let Some(dir) = cli.tickets_dir {
        PathBuf::from(dir)
    } else {
        find_tickets_dir(cli.repo_root.map(|p| p.to_string()))?
    };

    // Create ticket manager
    let mut manager = TicketManager::new(
        tickets_dir,
        cli.project.clone(),
        cli.category.clone(),
    );

    // Execute command
    cli.command.execute(&mut manager).await?;

    Ok(())
}
