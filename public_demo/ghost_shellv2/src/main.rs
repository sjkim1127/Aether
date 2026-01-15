use aether_ai::{anthropic, InjectionEngine, Template};
use aether_core::Slot;
use colored::*;
use dotenvy::dotenv;
use rhai::{Engine, ImmutableString, Scope};
use std::env;
use std::io::{self, Write};
use std::process::Command;

fn is_dangerous_command(cmd: &str) -> bool {
    let c = cmd.trim().to_lowercase();
    if c.is_empty() {
        return false;
    }

    let patterns = [
        "rm -rf",
        "rm -r -f",
        "rm -fr",
        "del /s",
        "del /q",
        "rmdir /s",
        "rd /s",
        "format ",
        "mkfs",
        "diskpart",
        "bcdedit",
        "bootrec",
        "shutdown",
        "reboot",
        "halt",
        "poweroff",
        "init 0",
        "init 6",
    ];

    patterns.iter().any(|p| c.contains(p))
}

fn run_command(cmd: &str) -> String {
    let output = if cfg!(target_os = "windows") {
        let command = format!("chcp 65001>nul & {}", cmd);
        Command::new("cmd").args(["/C", &command]).output()
    } else {
        Command::new("sh").args(["-c", cmd]).output()
    };

    match output {
        Ok(o) => {
            let mut out = String::from_utf8_lossy(&o.stdout).to_string();
            let err = String::from_utf8_lossy(&o.stderr).to_string();
            if !err.trim().is_empty() {
                if !out.trim().is_empty() {
                    out.push('\n');
                }
                out.push_str(&err);
            }
            if out.trim().is_empty() {
                "ok".to_string()
            } else {
                out
            }
        }
        Err(e) => format!("error: {}", e),
    }
}

fn confirm_prompt(prompt: &str) -> bool {
    print!("{} [y/N]: ", prompt);
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err() {
        return false;
    }
    matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
}

fn strip_code_fences(raw: &str) -> String {
    let trimmed = raw.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }

    let mut lines = Vec::new();
    for line in trimmed.lines() {
        if line.trim().starts_with("```") {
            continue;
        }
        lines.push(line);
    }
    lines.join("\n").trim().to_string()
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let api_key = env::var("ANTHROPIC_API_KEY").unwrap_or_default();
    if api_key.trim().is_empty() {
        eprintln!("{}", "ANTHROPIC_API_KEY is not set.".red());
        eprintln!("{}", "Set the key and try again.".red());
        return Ok(());
    }
    env::set_var("ANTHROPIC_API_KEY", &api_key);

    let model = env::var("ANTHROPIC_MODEL")
        .unwrap_or_else(|_| "claude-haiku-4-5-20251001".to_string());

    let provider = anthropic(&model)?;
    let aether = InjectionEngine::new(provider);

    let mut rhai_engine = Engine::new();

    rhai_engine.register_fn("print_color", |text: ImmutableString, color: ImmutableString| {
        let colored_text = match color.as_str() {
            "red" => text.red(),
            "green" => text.green(),
            "blue" => text.blue(),
            "yellow" => text.yellow(),
            "magenta" | "pink" => text.magenta(),
            "cyan" => text.cyan(),
            "bright" => text.bright_white(),
            _ => text.white(),
        };
        println!("{}", colored_text);
    });

    rhai_engine.register_fn("exec", |cmd: ImmutableString| -> String {
        let cmd = cmd.as_str();
        if is_dangerous_command(cmd) {
            return "DENIED: dangerous command (confirmation required)".to_string();
        }
        run_command(cmd)
    });

    rhai_engine.register_fn(
        "exec",
        |cmd: ImmutableString, allow_unsafe: bool| -> String {
            let cmd = cmd.as_str();
            if !allow_unsafe && is_dangerous_command(cmd) {
                return "DENIED: dangerous command (confirmation required)".to_string();
            }
            run_command(cmd)
        },
    );

    rhai_engine.register_fn("confirm", |prompt: ImmutableString| -> bool {
        confirm_prompt(prompt.as_str())
    });

    rhai_engine.register_fn("trim", |text: ImmutableString| -> String {
        text.trim().to_string()
    });
    rhai_engine.register_fn(
        "starts_with",
        |text: ImmutableString, prefix: ImmutableString| -> bool {
            text.starts_with(prefix.as_str())
        },
    );
    rhai_engine.register_fn(
        "contains",
        |text: ImmutableString, needle: ImmutableString| -> bool {
            text.contains(needle.as_str())
        },
    );
    rhai_engine.register_fn("substring", |text: ImmutableString, start: i64| -> String {
        let start = start.max(0) as usize;
        text.chars().skip(start).collect()
    });
    rhai_engine.register_fn(
        "substring",
        |text: ImmutableString, start: i64, len: i64| -> String {
            let start = start.max(0) as usize;
            let len = len.max(0) as usize;
            text.chars().skip(start).take(len).collect()
        },
    );
    rhai_engine.register_fn("to_lower", |text: ImmutableString| -> String {
        text.to_lowercase()
    });
    rhai_engine.register_fn("to_upper", |text: ImmutableString| -> String {
        text.to_uppercase()
    });
    rhai_engine.register_fn("len", |text: ImmutableString| -> i64 {
        text.chars().count() as i64
    });
    rhai_engine.register_fn("get_os", || -> String {
        env::consts::OS.to_string()
    });

    let mut mood = String::from("tsundere");
    let mut history: Vec<String> = Vec::new();

    println!("{}", "Ghost Shell v2 initialized.".cyan());
    println!("{}", "Type 'help' for commands, 'exit' to quit.".dimmed());

    loop {
        let cwd = env::current_dir()?;
        print!("ghost {} > ", cwd.display());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim().to_string();

        if input.is_empty() {
            continue;
        }

        if input == "exit" || input == "quit" {
            break;
        }

        if input == "help" {
            println!("{}", "Built-in commands:".cyan());
            println!("  help               show this message");
            println!("  mood <name>        change persona (tsundere, hacker, butler, paranoid)");
            println!("  history            show recent inputs");
            println!("  exit               quit");
            println!();
            println!("{}", "Environment:".cyan());
            println!("  ANTHROPIC_API_KEY  required");
            println!("  ANTHROPIC_MODEL    optional (default: claude-haiku-4-5-20251001)");
            continue;
        }

        if input == "history" {
            if history.is_empty() {
                println!("{}", "History is empty.".dimmed());
            } else {
                for (i, item) in history.iter().enumerate() {
                    println!("{:>2}: {}", i + 1, item);
                }
            }
            continue;
        }

        if let Some(new_mood) = input.strip_prefix("mood ") {
            mood = new_mood.trim().to_lowercase();
            println!("{}", format!("Mood set to: {}", mood).yellow());
            continue;
        }

        let history_text = if history.is_empty() {
            "none".to_string()
        } else {
            history
                .iter()
                .rev()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect::<Vec<_>>()
                .join("\n")
        };

        print!("{}", "thinking...".dimmed());
        io::stdout().flush()?;

        let system_prompt = format!(
            r#"You are Ghost Shell v2, an OS interface with a personality.

Current mood: {mood}
Current OS: {os}
Current dir: {cwd}
Recent history (newest last):
{history}
User input: "{input}"

Rules:
- Output ONLY raw Rhai code. No markdown, no prose.
- Use ONLY double quotes for strings.
- Do NOT declare variables already in scope: user_input, mood, current_os, current_dir, history.
- Do NOT use string methods (no ".starts_with", ".trim", etc). Use helper functions.
- Use print_color for all user-facing output.
- For dangerous commands (rm -rf, del /s, rmdir /s, format, mkfs, diskpart, shutdown),
  ALWAYS ask for confirmation and only then call exec(cmd, true).

Available API:
- print_color(text, color_name)
- exec(cmd) -> string (safe, blocks dangerous commands)
- exec(cmd, allow_unsafe) -> string
- confirm(prompt) -> bool
- get_os() -> string
- trim(text) -> string
- starts_with(text, prefix) -> bool
- contains(text, needle) -> bool
- substring(text, start) -> string
- substring(text, start, len) -> string
- to_lower(text) -> string
- to_upper(text) -> string
- len(text) -> int

Behavior:
- If input is a question, answer it with print_color.
- If input looks like a command, decide to run it or ask for clarification.
- Keep responses short and in-character."#,
            mood = mood,
            os = env::consts::OS,
            cwd = cwd.display(),
            history = history_text,
            input = input
        );

        let template = Template::new("{{AI:script}}")
            .configure_slot(Slot::new("script", &system_prompt).with_temperature(0.2));

        let generated_script = match aether.render(&template).await {
            Ok(s) => strip_code_fences(&s),
            Err(e) => {
                println!("{}", format!("\nerror: {}", e).red());
                continue;
            }
        };

        print!("\r           \r");

        let mut scope = Scope::new();
        scope.set_value("user_input", input.clone());
        scope.set_value("mood", mood.clone());
        scope.set_value("current_os", env::consts::OS.to_string());
        scope.set_value("current_dir", cwd.display().to_string());
        scope.set_value("history", history_text);

        if let Err(e) = rhai_engine.eval_with_scope::<()>(&mut scope, &generated_script) {
            println!("{}", format!("runtime error: {}", e).red());
            println!("script was:\n{}", generated_script.dimmed());
        }

        history.push(input);
        if history.len() > 20 {
            history.remove(0);
        }
    }

    Ok(())
}
