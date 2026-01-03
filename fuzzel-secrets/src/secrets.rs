use crate::secret::Secret;
use anyhow::{Context, Result};
use futures::future::try_join_all;
use oo7::{dbus::Service, AsAttributes};
use std::collections::HashMap;

const FUZZEL_SCHEMA: &str = "org.fuzzel.secrets";
const COLLECTION_LABEL: &str = "fuzzel-secrets";

/// Get all secrets from the fuzzel-secrets collection
pub async fn secrets() -> Result<Vec<String>> {
    let items = items().await?;
    let secrets_future = items.iter().map(|item| async {
        let item_label = item.label().await.context("Failed to get item label")?;
        Ok(item_label)
    });
    try_join_all(secrets_future).await
}

/// Get all unique field keys from all existing secrets
pub async fn all_field_keys() -> Result<Vec<String>> {
    let mut keys = std::collections::HashSet::new();

    for secret in secrets().await? {
        let data = get_data(&secret).await?;
        keys.extend(data.keys().cloned());
    }

    Ok(keys.into_iter().collect())
}

/// Store a secret
pub async fn store(label: &str, data: Secret) -> Result<()> {
    let json_data = serde_json::to_string(&data).context("Failed to serialize secret data")?;

    let attributes = HashMap::from([
        ("label".to_string(), label.to_string()), // Attributes are used by Secret Service for identification. Add label to ensure we store this as a separate item.
        ("xdg:schema".to_string(), FUZZEL_SCHEMA.to_string()),
    ]);

    fuzzel_collection()
        .await?
        .create_item(label, &attributes, json_data.as_bytes(), true, None)
        .await
        .context("Failed to create item")?;
    Ok(())
}

/// Get the data for a specific secret
pub async fn get_data(label: &str) -> Result<Secret> {
    let items = items_with_label(label).await?;
    match items.len() {
        0 => Err(anyhow::anyhow!(format!("Secret not found: {}", label))),
        1 => {
            let secret_data = items
                .first()
                .unwrap()
                .secret()
                .await
                .context("Failed to get secret")?;
            let json_str =
                String::from_utf8(secret_data.to_vec()).context("Failed to decode secret data")?;
            let data: Secret =
                serde_json::from_str(&json_str).context("Failed to parse secret data as JSON")?;
            return Ok(data);
        }
        _ => Err(anyhow::anyhow!(format!(
            "Multiple secrets found with label: {}",
            label
        ))),
    }
}

async fn items_with_label(label: &str) -> Result<Vec<oo7::dbus::Item<'static>>> {
    let mut attributes = HashMap::from([("xdg:schema", FUZZEL_SCHEMA)]);
    attributes.insert("label", label);
    search_items_by_attributes(attributes).await
}

async fn items() -> Result<Vec<oo7::dbus::Item<'static>>> {
    search_items_by_attributes(HashMap::from([("xdg:schema", FUZZEL_SCHEMA)])).await
}

async fn search_items_by_attributes(
    attributes: impl AsAttributes,
) -> Result<Vec<oo7::dbus::Item<'static>>> {
    fuzzel_collection()
        .await?
        .search_items(&attributes)
        .await
        .context("Failed to search items")
}

async fn fuzzel_collection() -> Result<oo7::dbus::Collection<'static>> {
    let service = Service::new()
        .await
        .context("Failed to connect to Secret Service")?;
    let collection = match service.with_label(COLLECTION_LABEL).await? {
        Some(col) => col,
        None => service
            .create_collection(COLLECTION_LABEL, "", None)
            .await
            .context("Failed to create fuzzel-secrets collection")?,
    };

    // Unlock the collection if it's locked
    if collection.is_locked().await? {
        collection
            .unlock(None)
            .await
            .context("Failed to unlock collection")?;
    }

    Ok(collection)
}
