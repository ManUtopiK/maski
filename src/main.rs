use clap::Parser;
use colored::*;
use std::process;

mod interactive;
mod types;

#[derive(Parser)]
#[command(name = "maski", version, about = "Interactive TUI for mask taskfiles")]
struct Cli {
    /// Path to a specific maskfile
    #[arg(long)]
    maskfile: Option<String>,

    /// Preview position: "down" (default), "right", "up", "left"
    #[arg(long, default_value = "down")]
    preview: String,
}

fn main() {
    let cli = Cli::parse();

    // Call mask --introspect to get the JSON AST
    let mut cmd = process::Command::new("mask");
    if let Some(ref path) = cli.maskfile {
        cmd.arg("--maskfile").arg(path);
    }
    cmd.arg("--introspect");

    let output = cmd.output().unwrap_or_else(|e| {
        eprintln!(
            "{} failed to run `mask --introspect`: {}",
            "ERROR:".red(),
            e
        );
        eprintln!("Is mask installed and in your PATH?");
        process::exit(1);
    });

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{} mask --introspect failed: {}", "ERROR:".red(), stderr);
        process::exit(1);
    }

    let json = String::from_utf8_lossy(&output.stdout);
    let maskfile: types::Maskfile = serde_json::from_str(&json).unwrap_or_else(|e| {
        eprintln!("{} failed to parse introspect JSON: {}", "ERROR:".red(), e);
        process::exit(1);
    });

    interactive::run(&maskfile.commands, &cli.maskfile, &cli.preview);
}
