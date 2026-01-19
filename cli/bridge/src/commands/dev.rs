use anyhow::{Context, Result};
use console::style;
use notify::{RecursiveMode, Watcher};

use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;

pub async fn handle(target: String, port: u16) -> Result<()> {
    println!(
        "{}",
        style(format!("Starting development server (target: {})", target))
            .bold()
            .cyan()
    );

    let project_root = std::env::current_dir()?;
    println!("  Watching: {}", project_root.display());

    println!("\n{}", style("Initial build...").cyan());
    build(&target)?;

    let (tx, rx) = channel();
    let mut watcher = notify::recommended_watcher(move |res| match res {
        Ok(event) => {
            let _ = tx.send(event);
        }
        Err(e) => println!("Watch error: {:?}", e),
    })?;

    watcher.watch(&project_root.join("src"), RecursiveMode::Recursive)?;
    watcher.watch(
        &project_root.join("Cargo.toml"),
        RecursiveMode::NonRecursive,
    )?;
    if project_root.join("python").exists() {
        watcher.watch(&project_root.join("python"), RecursiveMode::Recursive)?;
    }
    if project_root.join("nodejs").exists() {
        watcher.watch(&project_root.join("nodejs"), RecursiveMode::Recursive)?;
    }

    println!(
        "\n{}",
        style("👀 Watching for changes... (Press Ctrl+C to stop)").green()
    );
    if port != 3000 {
        println!(
            "  (Port argument {} ignored for now - auto-reload server not yet implemented)",
            port
        );
    }

    loop {
        match rx.recv() {
            Ok(_) => {
                // simple debounce: wait for a bit and drain channel
                std::thread::sleep(Duration::from_millis(100));
                while rx.try_recv().is_ok() {}

                println!("\n{}", style("File changed. Rebuilding...").cyan());
                if let Err(e) = build(&target) {
                    eprintln!("{} {}", style("❌ Build failed:").red(), e);
                } else {
                    println!("{}", style("√ Build successful!").green());
                }
            }
            Err(e) => println!("Watch error: {:?}", e),
        }
    }
}

fn build(target: &str) -> Result<()> {
    let mut args = vec!["run", "-p", "bridge", "--", "build"];
    if target != "all" {
        args.push("--target");
        args.push(target);
    }

    let status = Command::new("cargo")
        .args(&args)
        .status()
        .context("Failed to run bridge build")?;

    if status.success() {
        Ok(())
    } else {
        anyhow::bail!("Build command failed");
    }
}
