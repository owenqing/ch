use anyhow::Result;
use crossterm::{execute, terminal};
use std::io::Write;

pub fn execute_command(command: String) -> Result<()> {
    terminal::disable_raw_mode()?;
    execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;
    crossterm::terminal::disable_raw_mode()?;
    print!("\x1B[2J\x1B[1;1H");
    std::io::stdout().flush()?;
    println!("\n▶ Execute command:");
    println!("{}", command);
    println!("");
    println!("▶ Execute command result:");
    let output = std::process::Command::new("/bin/sh")
        .arg("-c")
        .arg(&command)
        .spawn()?
        .wait_with_output()?;
    if !output.stdout.is_empty() {
        println!("\n✅ Command output:");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if !output.stderr.is_empty() {
        eprintln!("\n❌ Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(())
} 