use colored::*;
use dialoguer::{Confirm, Input, Select};
use skim::prelude::*;
use std::process;
use std::sync::Arc;

use crate::types::Command;

struct ListItem {
    label: String,
    preview_text: String,
}

impl SkimItem for ListItem {
    fn text(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.label)
    }

    fn preview(&self, _context: PreviewContext) -> ItemPreview {
        ItemPreview::Text(self.preview_text.clone())
    }
}

fn pad_lines(text: &str, margin: usize) -> String {
    let pad = " ".repeat(margin);
    text.lines()
        .map(|line| format!("{}{}", pad, line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn build_preview(cmd: &Command) -> String {
    let mut preview = String::new();
    preview.push('\n'); // top margin

    if !cmd.description.is_empty() {
        preview.push_str(&format!("Description: {}\n", cmd.description));
    }

    if !cmd.subcommands.is_empty() {
        let subs: Vec<&str> = cmd.subcommands.iter().map(|s| s.name.as_str()).collect();
        preview.push_str(&format!("\nSubcommands: {}\n", subs.join(", ")));
    }

    if !cmd.required_args.is_empty() {
        let args: Vec<&str> = cmd.required_args.iter().map(|a| a.name.as_str()).collect();
        preview.push_str(&format!("\nRequired args: {}\n", args.join(", ")));
    }

    if !cmd.optional_args.is_empty() {
        let args: Vec<&str> = cmd.optional_args.iter().map(|a| a.name.as_str()).collect();
        preview.push_str(&format!("Optional args: {}\n", args.join(", ")));
    }

    let user_flags: Vec<_> = cmd
        .named_flags
        .iter()
        .filter(|f| f.name != "verbose")
        .collect();
    if !user_flags.is_empty() {
        preview.push_str("\nFlags:\n");
        for f in &user_flags {
            let flag_str = if f.short.is_empty() {
                format!("  --{}", f.long)
            } else {
                format!("  -{}, --{}", f.short, f.long)
            };
            if f.description.is_empty() {
                preview.push_str(&format!("{}\n", flag_str));
            } else {
                preview.push_str(&format!("{}  {}\n", flag_str, f.description));
            }
        }
    }

    if let Some(ref script) = cmd.script {
        preview.push_str(&format!("\nScript ({}):\n", script.executor));
        preview.push_str("─────────────\n");
        preview.push_str(&script.source);
        if !script.source.ends_with('\n') {
            preview.push('\n');
        }
    }

    pad_lines(&preview, 2)
}

fn build_items(commands: &[Command]) -> Vec<Arc<dyn SkimItem>> {
    commands
        .iter()
        .map(|cmd| {
            let label = if !cmd.subcommands.is_empty() {
                format!("{}  ▸", cmd.name)
            } else {
                cmd.name.clone()
            };
            Arc::new(ListItem {
                label,
                preview_text: build_preview(cmd),
            }) as Arc<dyn SkimItem>
        })
        .collect()
}

fn build_header(breadcrumb: &[String]) -> String {
    let nav_hint = "↑↓ navigate  → enter subcommands  ← back  Enter run  Esc quit";
    if breadcrumb.is_empty() {
        nav_hint.to_string()
    } else {
        format!("{}\n\n📂 {}", nav_hint, breadcrumb.join(" > "))
    }
}

fn run_skim(commands: &[Command], breadcrumb: &[String], preview_pos: &str) -> Option<(usize, Key)> {
    let items = build_items(commands);
    let header = build_header(breadcrumb);
    let preview_window = format!("{}:50%:wrap", preview_pos);

    let options = SkimOptionsBuilder::default()
        .height(Some("100%"))
        .preview(Some(""))
        .preview_window(Some(&preview_window))
        .multi(false)
        .expect(Some("right,left,enter".to_string()))
        .prompt(Some("  Command> "))
        .header(Some(&header))
        .build()
        .expect("failed to build skim options");

    let (tx, rx): (SkimItemSender, SkimItemReceiver) = unbounded();
    for item in items {
        tx.send(item).unwrap();
    }
    drop(tx);

    let output = Skim::run_with(&options, Some(rx));

    match output {
        Some(out) if !out.is_abort && !out.selected_items.is_empty() => {
            let selected_text = out.selected_items[0].output().to_string();
            let index = commands.iter().enumerate().find_map(|(i, cmd)| {
                let label = if !cmd.subcommands.is_empty() {
                    format!("{}  ▸", cmd.name)
                } else {
                    cmd.name.clone()
                };
                if label == selected_text.trim_end() {
                    Some(i)
                } else {
                    None
                }
            });

            index.map(|i| (i, out.final_key))
        }
        _ => None,
    }
}

/// Prompt for arguments and flags, returns (positional_args, flag_args) for the mask command line.
fn prompt_arguments(cmd: &Command) -> (Vec<String>, Vec<String>) {
    let mut positional = Vec::new();
    let mut flags = Vec::new();

    // Required args
    for arg in &cmd.required_args {
        let val: String = Input::new()
            .with_prompt(&arg.name)
            .interact_text()
            .unwrap_or_else(|_| process::exit(0));
        positional.push(val);
    }

    // Optional args
    for arg in &cmd.optional_args {
        let val: String = Input::new()
            .with_prompt(format!("{} (optional)", arg.name))
            .allow_empty(true)
            .interact_text()
            .unwrap_or_else(|_| process::exit(0));
        if !val.is_empty() {
            positional.push(val);
        }
    }

    // Named flags
    for flag in &cmd.named_flags {
        if flag.name == "verbose" {
            continue;
        }

        if !flag.choices.is_empty() {
            let selection = Select::new()
                .with_prompt(&flag.name)
                .items(&flag.choices)
                .default(0)
                .interact_opt()
                .unwrap_or_else(|_| process::exit(0));
            if let Some(idx) = selection {
                flags.push(format!("--{}", flag.long));
                flags.push(flag.choices[idx].clone());
            }
        } else if flag.takes_value {
            let prompt_label = if flag.description.is_empty() {
                flag.name.clone()
            } else {
                format!("{} ({})", flag.name, flag.description)
            };
            let val: String = Input::new()
                .with_prompt(prompt_label)
                .allow_empty(!flag.required)
                .interact_text()
                .unwrap_or_else(|_| process::exit(0));

            if !val.is_empty() {
                if flag.validate_as_number
                    && val.parse::<isize>().is_err()
                    && val.parse::<f32>().is_err()
                {
                    eprintln!(
                        "{} flag `{}` expects a numerical value",
                        "ERROR:".red(),
                        flag.name
                    );
                    process::exit(1);
                }
                flags.push(format!("--{}", flag.long));
                flags.push(val);
            }
        } else {
            let enabled = Confirm::new()
                .with_prompt(&flag.name)
                .default(false)
                .interact_opt()
                .unwrap_or_else(|_| process::exit(0));
            if enabled == Some(true) {
                flags.push(format!("--{}", flag.long));
            }
        }
    }

    (positional, flags)
}

fn execute(cmd: &Command, breadcrumb: &[String], maskfile_arg: &Option<String>) {
    let has_prompts = !cmd.required_args.is_empty()
        || !cmd.optional_args.is_empty()
        || cmd.named_flags.iter().any(|f| f.name != "verbose");

    // Build the full command path: breadcrumb + command name
    let mut cmd_parts: Vec<&str> = breadcrumb.iter().map(|s| s.as_str()).collect();
    cmd_parts.push(&cmd.name);
    let display_path = cmd_parts.join(" > ");

    let (positional, flags) = if has_prompts {
        println!("\n{} {}\n", "Selected:".green().bold(), display_path.bold());
        prompt_arguments(cmd)
    } else {
        (vec![], vec![])
    };

    println!("\n{} {}\n", "Running:".cyan().bold(), display_path.bold());

    // Build mask subprocess command
    let mut child = process::Command::new("mask");

    if let Some(ref path) = maskfile_arg {
        child.arg("--maskfile").arg(path);
    }

    // Add the command path (subcommands as separate args)
    for part in &cmd_parts {
        child.arg(part);
    }

    // Add flags then positional args
    for flag in &flags {
        child.arg(flag);
    }
    for arg in &positional {
        child.arg(arg);
    }

    match child.status() {
        Ok(status) => {
            process::exit(status.code().unwrap_or(0));
        }
        Err(err) => {
            eprintln!("{} {}", "ERROR:".red(), err);
            process::exit(1);
        }
    }
}

pub fn run(commands: &[Command], maskfile_arg: &Option<String>, preview_pos: &str) {
    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        eprintln!("{} interactive mode requires a TTY", "ERROR:".red());
        process::exit(1);
    }

    if commands.is_empty() {
        eprintln!("No executable commands found in maskfile");
        process::exit(1);
    }

    let mut stack: Vec<(&[Command], Vec<String>)> = vec![];
    let mut current_commands: &[Command] = commands;
    let mut breadcrumb: Vec<String> = vec![];

    loop {
        let result = run_skim(current_commands, &breadcrumb, preview_pos);

        let (index, key) = match result {
            Some(r) => r,
            None => {
                // Esc: go back or quit
                if let Some((prev_commands, prev_breadcrumb)) = stack.pop() {
                    current_commands = prev_commands;
                    breadcrumb = prev_breadcrumb;
                    continue;
                }
                return;
            }
        };

        let cmd = &current_commands[index];

        match key {
            Key::Left => {
                if let Some((prev_commands, prev_breadcrumb)) = stack.pop() {
                    current_commands = prev_commands;
                    breadcrumb = prev_breadcrumb;
                }
            }
            Key::Right => {
                if !cmd.subcommands.is_empty() {
                    stack.push((current_commands, breadcrumb.clone()));
                    breadcrumb.push(cmd.name.clone());
                    current_commands = &current_commands[index].subcommands;
                }
            }
            Key::Enter | _ => {
                if cmd.script.is_some() {
                    execute(cmd, &breadcrumb, maskfile_arg);
                    return;
                } else if !cmd.subcommands.is_empty() {
                    stack.push((current_commands, breadcrumb.clone()));
                    breadcrumb.push(cmd.name.clone());
                    current_commands = &current_commands[index].subcommands;
                }
            }
        }
    }
}
