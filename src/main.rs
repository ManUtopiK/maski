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

    /// Task and arguments to forward to `mask` (interactive mode if empty)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    args: Vec<String>,
}

fn main() {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        // Hand `--help` over to mask so the task list shows up, then append
        // what maski adds on top of it.
        Err(err) if err.kind() == clap::error::ErrorKind::DisplayHelp => {
            print_help();
            process::exit(0);
        }
        Err(err) => err.exit(),
    };

    // `maski help` is the same request as `maski --help`. Anything more
    // specific (`maski help build`) belongs to mask alone.
    if cli.args == ["help"] {
        print_help();
        process::exit(0);
    }

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

    // Arguments naming a command that only holds subcommands open the TUI at
    // that level; anything else (a runnable task, flags, unknown names) goes
    // straight to mask.
    if !cli.args.is_empty() && resolve_group(&maskfile.commands, &cli.args).is_none() {
        passthrough(&cli.maskfile, &cli.args);
    }

    // Read maskfile.md to extract full markdown sections
    let maskfile_path = find_maskfile(&cli.maskfile).unwrap_or_else(|| {
        eprintln!("{} no maskfile.md found", "ERROR:".red());
        process::exit(1);
    });
    let maskfile_content = std::fs::read_to_string(&maskfile_path).unwrap_or_default();
    let sections = maskfile_reader::extract_sections(&maskfile_content);

    interactive::run(
        &maskfile.commands,
        &cli.maskfile,
        &cli.preview,
        &sections,
        &cli.args,
    );
}

/// Walk `args` down the command tree. Returns the matched path only when it
/// lands on a group — a command with subcommands and no script of its own.
fn resolve_group<'a>(commands: &'a [types::Command], args: &[String]) -> Option<&'a types::Command> {
    let mut level = commands;
    let mut found = None;

    for arg in args {
        let cmd = level.iter().find(|c| &c.name == arg)?;
        level = &cmd.subcommands;
        found = Some(cmd);
    }

    found.filter(|cmd| cmd.script.is_none() && !cmd.subcommands.is_empty())
}

/// Print `mask --help` — task list included — followed by maski's own additions.
fn print_help() {
    let mut cmd = process::Command::new("mask");
    if let Some(ref path) = maskfile_from_raw_args() {
        cmd.arg("--maskfile").arg(path);
    }
    cmd.arg("--help");

    if cmd.status().is_err() {
        eprintln!("{} failed to run `mask --help`", "ERROR:".red());
        eprintln!("Is mask installed and in your PATH?");
    }

    // Same two-column layout as mask's own sections. Pad before coloring:
    // the escape codes would otherwise count towards the width.
    let row = |item: &str, desc: &str| {
        let pad = " ".repeat(29_usize.saturating_sub(item.len()));
        println!("    {}{}{}", item.green(), pad, desc);
    };

    println!("\n{}", "MASKI OPTIONS:".cyan());
    row(
        "    --preview <position>",
        "Preview panel position: down (default), right, up, left",
    );

    println!("\n{}", "MASKI USAGE:".cyan());
    row("maski", "Browse tasks interactively");
    row("maski <group>", "Browse a group's subcommands, e.g. `maski db`");
    row("maski <task> [args]...", "Forwarded to mask, like everything else");
}

/// Pull `--maskfile <path>` out of the raw argv. The help path runs before clap
/// has produced a parsed `Cli`, so it cannot read it from there.
fn maskfile_from_raw_args() -> Option<String> {
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        if let Some(path) = arg.strip_prefix("--maskfile=") {
            return Some(path.to_string());
        }
        if arg == "--maskfile" {
            return args.next();
        }
    }
    None
}

/// Forward every argument to `mask`, inheriting stdio and exit code.
fn passthrough(maskfile: &Option<String>, args: &[String]) -> ! {
    let mut cmd = process::Command::new("mask");
    if let Some(ref path) = maskfile {
        cmd.arg("--maskfile").arg(path);
    }
    cmd.args(args);

    match cmd.status() {
        Ok(status) => process::exit(status.code().unwrap_or(0)),
        Err(e) => {
            eprintln!("{} failed to run `mask`: {}", "ERROR:".red(), e);
            eprintln!("Is mask installed and in your PATH?");
            process::exit(1);
        }
    }
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
