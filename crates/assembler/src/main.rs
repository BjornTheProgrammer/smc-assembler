use anyhow::Result;
use clap::{Parser as ClapParser, Subcommand};
use smc_assembler::{assembler::backends::Backend, compile_to_file, save::memory::Format};
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

        /// Target backend
        #[arg(short, long)]
        target: Backend,

        /// Instruction memory format for schematic file
        #[arg(short, long)]
        format: Option<Format>,

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
            target,
            debug_artifacts,
            format,
        } => compile_to_file(input, output, target.clone(), *debug_artifacts, *format)?,
    }

    Ok(())
}
