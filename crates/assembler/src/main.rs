use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use smc_assembler::compile_to_file;
use tracing::instrument;

#[derive(ClapParser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Compiles the given source file
    Compile {
        /// Path to the input file
        input: String,

        /// Path of the output file
        output: String,

        /// Generate debug artifacts
        #[arg(long)]
        debug_artifacts: bool,
    },
}

#[instrument]
fn main() -> Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Compile {
            input,
            output,
            debug_artifacts,
        } => compile_to_file(input, output, *debug_artifacts)?,
    }

    Ok(())
}
