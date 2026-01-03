use anyhow::{Context, Result};
use std::io::Write;
use std::process::{Command, Stdio};

/// Let the user provide a password without any suggestions
pub fn request_password(placeholder: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("fuzzel");
    cmd.args(["--dmenu", "--lines", "0"]);

    if let Some(ph) = placeholder {
        cmd.arg("--placeholder").arg(ph);
    }

    cmd.arg("--password");

    let output = cmd.output().context("Failed to execute fuzzel")?;

    if !output.status.success() {
        anyhow::bail!("fuzzel command failed");
    }

    let result = String::from_utf8(output.stdout).context("Failed to parse fuzzel output")?;

    Ok(result.trim().to_string())
}

/// Let the user provide input without any suggestions
pub fn request_input(placeholder: Option<&str>) -> Result<String> {
    let mut cmd = Command::new("fuzzel");
    cmd.args(["--dmenu", "--lines", "0"]);

    if let Some(ph) = placeholder {
        cmd.arg("--placeholder").arg(ph);
    }

    let output = cmd.output().context("Failed to execute fuzzel")?;

    if !output.status.success() {
        anyhow::bail!("fuzzel command failed");
    }

    let result = String::from_utf8(output.stdout).context("Failed to parse fuzzel output")?;

    Ok(result.trim().to_string())
}

/// Select one of the items with Fuzzel, returning the index
pub fn select_index(items: &[String], placeholder: Option<&str>) -> Result<usize> {
    let input_data = items.join("\n");

    let mut cmd = Command::new("fuzzel");
    cmd.args(["--dmenu", "--index"]);

    if let Some(ph) = placeholder {
        cmd.arg("--placeholder").arg(ph);
    }

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn fuzzel")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_data.as_bytes())
            .context("Failed to write to fuzzel stdin")?;
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for fuzzel")?;

    if !output.status.success() {
        anyhow::bail!("fuzzel command failed");
    }

    let result = String::from_utf8(output.stdout).context("Failed to parse fuzzel output")?;

    result
        .trim()
        .parse::<usize>()
        .context("Failed to parse index")
}

/// Select one of the items with Fuzzel
pub fn select(items: &[String], placeholder: Option<&str>) -> Result<String> {
    let index = select_index(items, placeholder)?;
    items
        .get(index)
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("Invalid index returned from fuzzel"))
}

/// Select from suggestions or type a new value
pub fn select_or_input(items: &[String], placeholder: Option<&str>) -> Result<String> {
    let input_data = items.join("\n");

    let mut cmd = Command::new("fuzzel");
    cmd.arg("--dmenu");

    if let Some(ph) = placeholder {
        cmd.arg("--placeholder").arg(ph);
    }

    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());

    let mut child = cmd.spawn().context("Failed to spawn fuzzel")?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_data.as_bytes())
            .context("Failed to write to fuzzel stdin")?;
    }

    let output = child
        .wait_with_output()
        .context("Failed to wait for fuzzel")?;

    if !output.status.success() {
        anyhow::bail!("fuzzel command failed");
    }

    let result = String::from_utf8(output.stdout).context("Failed to parse fuzzel output")?;

    Ok(result.trim().to_string())
}
