use crate::output::formatter;
use anyhow::Result;
use clap::{Args, Subcommand};
use jules_api::JulesClient;

#[derive(Subcommand)]
pub enum SourcesCommand {
    /// List all available sources
    List(ListSourcesArgs),
    /// Get a specific source by ID
    Get(GetSourceArgs),
}

#[derive(Args)]
pub struct ListSourcesArgs {
    /// Filter sources (e.g., 'name=sources/source1 OR name=sources/source2')
    #[arg(short, long)]
    filter: Option<String>,
    #[arg(short, long)]
    page_size: Option<u32>,
    #[arg(short = 't', long)]
    page_token: Option<String>,
}

#[derive(Args)]
pub struct GetSourceArgs {
    /// The ID of the source to retrieve
    #[arg(required = true)]
    pub source_id: String,
}

pub async fn handle_sources_command(
    command: &SourcesCommand,
    client: &JulesClient,
    format: &str,
) -> Result<()> {
    match command {
        SourcesCommand::List(args) => {
            let response = client
                .list_sources(args.page_size, args.page_token.as_deref(), args.filter.as_deref())
                .await?;
            formatter::print_sources_response(&response, format)?;
        }
        SourcesCommand::Get(args) => {
            let source = client.get_source(&args.source_id).await?;
            formatter::print_source(&source, format)?;
        }
    }
    Ok(())
}