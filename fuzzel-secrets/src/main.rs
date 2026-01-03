use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use fuzzel_secrets::{field::Field, fuzzel, secret::Secret, secrets};
use std::process::Command;

#[derive(Parser)]
#[command(name = "fuzzel-secrets")]
#[command(about = "Manage secrets with fuzzel interface", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Store a new secret
    Store,
    /// Retrieve and type a secret
    Retrieve,
}

async fn retrieve() -> Result<()> {
    let mut all_secrets = secrets::secrets()
        .await
        .context("Failed to retrieve secrets")?;
    all_secrets.sort();
    let all_secrets = all_secrets;

    let requested_secret =
        fuzzel::select(&all_secrets, Some("Select secret")).context("Failed to select secret")?;

    let data = secrets::get_data(&requested_secret)
        .await
        .context("Failed to get secret data")?;

    if data.is_empty() {
        return Err(anyhow::anyhow!("No fields found in secret"));
    }

    let fields: Vec<String> = data.keys().cloned().collect();
    let field = fuzzel::select(&fields, Some("Field")).context("Failed to select field")?;

    let value = data
        .get(&field)
        .ok_or_else(|| anyhow::anyhow!("Field not found"))?;

    Command::new("wtype")
        .arg("--")
        .arg(&value.value)
        .status()
        .context("Failed to execute wtype")?;

    Ok(())
}

async fn store() -> Result<()> {
    // Start fetching existing keys concurrently, it takes some time
    let existing_keys_task = tokio::spawn(async move { secrets::all_field_keys().await });

    let mut all_secrets = secrets::secrets()
        .await
        .context("Failed to retrieve secrets")?;
    all_secrets.sort();
    let all_secrets = all_secrets;

    let requested_secret = fuzzel::select_or_input(&all_secrets, Some("Label"))
        .context("Failed to get service name")?;

    // Check if service already exists and load its data
    let existing_secret = all_secrets
        .iter()
        .find(|secret| secret.to_string() == requested_secret);

    let mut data = match existing_secret {
        Some(secret) => secrets::get_data(secret)
            .await
            .context("Failed to get existing secret data")?,
        None => Secret::new(),
    };

    let mut existing_keys = existing_keys_task
        .await
        .context("Failed to join existing keys task")?
        .context("Failed to retrieve existing field keys")?;
    existing_keys.sort();
    let existing_keys = existing_keys;

    let add_field_option = "+   Add field";
    let complete_option = "✓   Complete";

    loop {
        // Build menu items
        let mut menu_items: Vec<String> =
            vec![add_field_option.to_string(), complete_option.to_string()];

        let mut field_items: Vec<String> = data.iter().map(|f| f.display()).collect();
        field_items.sort();

        menu_items.extend(field_items);
        let menu_items = menu_items;

        let selection = fuzzel::select(&menu_items, Some("Secret fields"))
            .context("Failed to select menu item")?;

        if selection == complete_option {
            break;
        } else if selection == add_field_option {
            // Add new field
            let key = fuzzel::select_or_input(&existing_keys, Some("Field name"))
                .context("Failed to get field name")?;

            let request_input_fn = if Field::is_key_sensitive(&key) {
                fuzzel::request_password
            } else {
                fuzzel::request_input
            };
            let value = request_input_fn(Some(&format!("Value for '{}'", key)))
                .context("Failed to get field value")?;

            data.insert(key, value);
        } else {
            // Edit existing field
            let field = Field::parse_from_display(&selection)?;
            let request_input_fn = if field.is_sensitive() {
                fuzzel::request_password
            } else {
                fuzzel::request_input
            };
            let value = request_input_fn(Some(&format!("New value for '{}'", field.key)))
                .context("Failed to get field value")?;

            data.insert(field.key, value);
        }
    }

    if data.is_empty() {
        return Err(anyhow::anyhow!("No fields provided"));
    }

    secrets::store(&requested_secret, data)
        .await
        .context("Failed to store secret")?;

    println!("Secret stored successfully");

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Store => store().await?,
        Commands::Retrieve => retrieve().await?,
    }

    Ok(())
}
