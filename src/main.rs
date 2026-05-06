mod cli;
mod executor;
mod output;
mod tracer;
mod types;
mod utils;

use clap::Parser;
use cli::{Cli, Commands};
use eyre::{Ok, Result};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::TraceLocal { 
            contract, 
            calldata, 
            from, 
            to, 
            value, 
            gas_limit, 
            json, 
            max_steps 
        } => {
            let output = executor::trace_local(
                &contract, 
                &calldata, 
                &from, 
                &to, 
                &value, 
                gas_limit, 
                max_steps
            )?;

            output::print_summary(&output);

            if let Some(path) = json {
                output::write_json(&path, &output)?;
                println!("trace written to {}", path.display());
            }            
        }
    }
    
    Ok(())
}