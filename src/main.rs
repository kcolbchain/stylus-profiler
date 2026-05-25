use clap::{Parser, Subcommand};
use colored::Colorize;
use std::path::PathBuf;

mod analyzer;
mod compare;
mod display;
mod gas;
mod size;

#[derive(Parser)]
#[command(name = "stylus-profiler", about = "WASM binary analyzer for Arbitrum Stylus contracts")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Full analysis of a Stylus WASM binary
    Analyze {
        /// Path to the WASM file
        wasm: PathBuf,
        /// Output format: table or json
        #[arg(long, default_value = "table")]
        format: String,
    },
    /// Size breakdown by function and module
    Size {
        /// Path to the WASM file
        wasm: PathBuf,
        /// Show top N functions by size
        #[arg(short = 'n', long, default_value = "20")]
        top: usize,
    },
    /// Compare two WASM builds for regressions
    Compare {
        /// Old WASM binary
        old: PathBuf,
        /// New WASM binary
        new: PathBuf,
    },
    /// Suggest optimizations
    Optimize {
        /// Path to the WASM file
        wasm: PathBuf,
    },
}

fn main() -> eyre::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze { wasm, format } => {
            let analysis = analyzer::analyze(&wasm)?;
            match format.as_str() {
                "json" => {
                    let report = analysis.to_report();
                    println!("{}", serde_json::to_string_pretty(&report)?);
                }
                _ => {
                    println!("{}", "stylus-profiler".bold().cyan());
                    println!("{}", "─".repeat(50).dimmed());
                    display::print_analysis(&analysis);
                }
            }
        }
        Commands::Size { wasm, top } => {
            let analysis = analyzer::analyze(&wasm)?;
            display::print_size_breakdown(&analysis, top);
        }
        Commands::Compare { old, new } => {
            let old_a = analyzer::analyze(&old)?;
            let new_a = analyzer::analyze(&new)?;
            compare::print_comparison(&old_a, &new_a);
        }
        Commands::Optimize { wasm } => {
            let analysis = analyzer::analyze(&wasm)?;
            display::print_optimizations(&analysis);
        }
    }

    Ok(())
}
