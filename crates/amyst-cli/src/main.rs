use amyst_core::Engine;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs an Amyst script
    Run {
        /// Script to be executed
        script: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Run { script } => {
            let mut engine = Engine::new();
            if let Err(err_message) = engine.run(script) {
                eprintln!("{err_message}");
                std::process::exit(70);
            }
        }
    }
}
