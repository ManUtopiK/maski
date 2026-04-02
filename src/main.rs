use clap::Parser;
use colored::*;
use std::process;

mod interactive;
mod maskfile_reader;
mod md4x;
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

    // Read maskfile.md to extract full markdown sections
    let maskfile_path = find_maskfile(&cli.maskfile).unwrap_or_else(|| {
        eprintln!("{} no maskfile.md found", "ERROR:".red());
        process::exit(1);
    });
    let maskfile_content = std::fs::read_to_string(&maskfile_path).unwrap_or_default();
    let sections = maskfile_reader::extract_sections(&maskfile_content);

    interactive::run(&maskfile.commands, &cli.maskfile, &cli.preview, &sections);
}

fn find_maskfile(explicit: &Option<String>) -> Option<String> {
    if let Some(ref path) = explicit {
        return Some(path.clone());
    }
    let mut dir = std::env::current_dir().ok()?;
    loop {
        for name in &["maskfile.md", "Maskfile.md"] {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate.to_string_lossy().into_owned());
            }
        }
        if !dir.pop() {
            return None;
        }
    }
}
