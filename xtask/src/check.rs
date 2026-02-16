use std::process::{Command, Stdio};

pub fn run() -> anyhow::Result<()> {
    let checks = [
        ("Format check", vec!["cargo", "fmt", "--all", "--", "--check"]),
        (
            "Clippy",
            vec!["cargo", "clippy", "--all-targets", "--all-features", "--", "-W", "warnings"],
        ),
        ("Tests", vec!["cargo", "test", "--all"]),
        ("Dependency audit", vec!["cargo", "deny", "check"]),
    ];

    for (name, cmd) in checks {
        println!("\n▶ {name}");
        let status = if cfg!(target_os = "windows") {
            Command::new("cmd").args(["/C", &cmd.join(" ")]).status()?
        } else {
            Command::new(cmd[0])
                .args(&cmd[1..])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()?
        };

        if !status.success() {
            eprintln!("\n✗ {name} failed");
            anyhow::bail!("{name} failed");
        }
        println!("✓ {name} passed");
    }

    println!("\n✓ All checks passed!");
    Ok(())
}
