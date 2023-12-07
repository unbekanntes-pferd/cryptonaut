use dco3::{auth::Connected, nodes::NodeType, Dracoon, Nodes, OAuth2Flow, RescueKeyPair};
use tracing::{error, info};

use self::models::{CryptoNautError, CryptoNautConfig};

pub mod models;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "|", env!("CARGO_PKG_VERSION"));

pub async fn distribute_missing_keys(path: String, config: &CryptoNautConfig) -> Result<(), CryptoNautError> {
    let (client, path) = get_client_and_path(path, config).await?;

    let (node_id, node_type) = get_node_info(&client, path).await?;

    let rescue_key_secret = config.get_rescue_key();

    match (node_id, node_type) {
        (0, _) => {
            info!("Distributing missing keys for all nodes.");
            distribute_all_keys(&client, &rescue_key_secret, None, None).await?;
        }
        (_, NodeType::Room) | (_, NodeType::Folder) => {
            info!(
                "Distributing missing keys for room with node id {}.",
                node_id
            );

            let node_id = Some(node_id);

           distribute_all_keys(&client, rescue_key_secret, node_id, None).await?;
        }
        (_, NodeType::File) => {
            info!(
                "Distributing missing keys for file with node id {}.",
                node_id
            );

            let node_id = Some(node_id);

            distribute_all_keys(&client, &rescue_key_secret, None, node_id).await?;

        }
    };

    info!("All missing keys distributed.");

    Ok(())
}

async fn distribute_all_keys(client: &Dracoon<Connected>, rescue_key_secret: &str, room_id: Option<u64>, file_id: Option<u64>) -> Result<(), CryptoNautError> { 

    let mut missing_keys = client
        .distribute_missing_keys(&rescue_key_secret, room_id, file_id, None)
        .await
        .map_err(|err| {
            error!("Error: {}", err);
            err
        })?;

    let distributed_keys = if missing_keys > 100 {
        100
    } else {
        missing_keys
    };

    info!("{} missing keys distributed.", distributed_keys);

    while missing_keys > 100 {
        info!("More keys found - fetching again");
        missing_keys = client
            .distribute_missing_keys(&rescue_key_secret, room_id, file_id, None)
            .await
            .map_err(|err| {
                error!("Error: {}", err);
                err
            })?;

        let distributed_keys = if missing_keys > 100 {
            100
        } else {
            missing_keys
        };

        info!("{} missing keys distributed.", distributed_keys);
    }

    Ok(())

}

async fn get_node_info(
    dracoon: &Dracoon<Connected>,
    path: String,
) -> Result<(u64, NodeType), CryptoNautError> {
    // Return early if the path is root.
    if path == "/" {
        return Ok((0, NodeType::Room));
    }

    // Fetch the node from the path.
    let node_option = dracoon.get_node_from_path(&path).await?;

    // Check if the node exists.
    if let Some(node) = node_option {
        // Determine node_id based on the node type.
        let node_id = match node.node_type {
            NodeType::Folder => node.auth_parent_id.unwrap_or(0),
            _ => node.id,
        };

        return Ok((node_id, node.node_type));
    }

    // Default case when node is None.
    Ok((0, NodeType::Room))
}

async fn get_client_and_path(
    path: impl Into<String>,
    config: &CryptoNautConfig,
) -> Result<(Dracoon<Connected>, String), CryptoNautError> {
    let refresh_token =
        config
            .get_refresh_token();

    let client_id = config.get_client_id();
    let client_secret = config.get_client_secret();

    let (base_url, path) = split_url(&path.into())?;

    let dracoon = Dracoon::builder()
        .with_base_url(base_url)
        .with_client_id(client_id)
        .with_client_secret(client_secret)
        .with_user_agent(USER_AGENT)
        .build()
        .map_err(|err| CryptoNautError::Http(err))?
        .connect(OAuth2Flow::refresh_token(refresh_token))
        .await?;

    Ok((dracoon, path))
}


fn split_url(url: &str) -> Result<(String, String), CryptoNautError> {
    if url.starts_with("http://") {
        return Err(CryptoNautError::InvalidUrl(url.to_string()));
    }

    let stripped_url = url.strip_prefix("https://").unwrap_or(url);

    if let Some(index) = stripped_url.find('/') {
        let (base_url, path) = stripped_url.split_at(index);

        let base_url = format!("https://{}", base_url);
        Ok((base_url.to_string(), path.to_string()))
    } else {
        let stripped_url = format!("https://{}", stripped_url);
        Ok((stripped_url.to_string(), "/".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::{split_url, CryptoNautError};

    #[test]
    fn test_with_https_and_path() {
        assert_eq!(
            split_url("https://some.url.domain/some/path"),
            Ok((
                "https://some.url.domain".to_string(),
                "/some/path".to_string()
            ))
        );
    }

    #[test]
    fn test_without_https_with_path() {
        assert_eq!(
            split_url("some.url.domain/some/path"),
            Ok((
                "https://some.url.domain".to_string(),
                "/some/path".to_string()
            ))
        );
    }

    #[test]
    fn test_with_https_no_path() {
        assert_eq!(
            split_url("https://some.url.domain"),
            Ok(("https://some.url.domain".to_string(), "/".to_string()))
        );
    }

    #[test]
    fn test_without_https_no_path() {
        assert_eq!(
            split_url("some.url.domain"),
            Ok(("https://some.url.domain".to_string(), "/".to_string()))
        );
    }

    #[test]
    fn test_empty_string() {
        assert_eq!(split_url(""), Ok(("https://".to_string(), "/".to_string())));
    }

    #[test]
    fn test_only_path() {
        assert_eq!(
            split_url("/some/path"),
            Ok(("https://".to_string(), "/some/path".to_string()))
        );
    }

    #[test]
    fn test_http_prefix() {
        assert_eq!(
            split_url("http://some.url.domain"),
            Err(CryptoNautError::InvalidUrl(
                "http://some.url.domain".to_string()
            ))
        );
    }
}
