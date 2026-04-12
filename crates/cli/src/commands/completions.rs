use anyhow::{Result, bail};
use clap::CommandFactory;
use clap_complete::{Shell, generate};
use std::io;

pub fn generate_completions(shell_name: &str) -> Result<()> {
    let mut cmd = crate::Cli::command();
    match shell_name {
        "bash" => generate(Shell::Bash, &mut cmd, "plinth", &mut io::stdout()),
        "zsh" => generate(Shell::Zsh, &mut cmd, "plinth", &mut io::stdout()),
        "fish" => generate(Shell::Fish, &mut cmd, "plinth", &mut io::stdout()),
        "elvish" => generate(Shell::Elvish, &mut cmd, "plinth", &mut io::stdout()),
        "nushell" => generate(
            clap_complete_nushell::Nushell,
            &mut cmd,
            "plinth",
            &mut io::stdout(),
        ),
        _ => bail!("Unsupported shell '{shell_name}'. Supported: bash, zsh, fish, elvish, nushell"),
    }
    Ok(())
}
