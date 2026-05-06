use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "mini-evm-executor-tracer")]
#[command(about = "A minimal EVM execution tracer based on revm")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    TraceLocal {
        #[arg(long)]
        contract: PathBuf,

        #[arg(long, default_value = "0x")]
        calldata: String,

        #[arg(long)]
        from: String,

        #[arg(long)]
        to: String,

        #[arg(long, default_value = "0")]
        value: String,

        #[arg(long, default_value = "30000000")]
        gas_limit: u64,

        #[arg(long)]
        json: Option<PathBuf>,

        #[arg(long)]
        max_steps: Option<usize>,
    },
}