use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use tokio::fs;
#[cfg(not(feature = "ai"))]
use tracing::warn;
use tracing::{info, instrument};

#[cfg(feature = "ai")]
use hf_hub::{api::tokio::Api, Repo, RepoType};

/// Represents the status of a model in the local system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ModelStatus {
    /// The unique name or identifier for the model.
    pub name: String,
    /// The size of the model in megabytes, if available.
    pub size_mb: Option<f64>,
    /// Whether the model is currently loaded into memory.
    pub loaded: bool,
    /// The local filesystem path to the model, if it exists.
    pub path: Option<String>,
}

/// Manages the lifecycle of AI models: downloading, caching, and loading.
pub struct ModelManager {
    models_dir: PathBuf,
}

impl ModelManager {
    /// Creates a new ModelManager with a specified storage directory.
    pub fn new(storage_path: &Path) -> Self {
        let models_dir = storage_path.join("models");
        Self { models_dir }
    }

    /// Initializes the model manager, ensuring the storage directory exists.
    #[instrument(skip(self))]
    pub async fn init(&self) -> Result<()> {
        if !self.models_dir.exists() {
            info!("Creating models directory at {:?}", self.models_dir);
            fs::create_dir_all(&self.models_dir)
                .await
                .context("Failed to create models directory")?;
        }
        Ok(())
    }

    /// Checks if a specific model is cached locally.
    ///
    /// For now, we check if the directory for the model exists and is not empty.
    pub fn is_cached(&self, model_name: &str) -> bool {
        // Sanitize model name to be a valid directory name (replace / with _)
        let safe_name = model_name.replace('/', "_");
        let model_path = self.models_dir.join(&safe_name);

        // Simple check: does directory exist?
        // A more robust check would verify specific files (config.json, model.safetensors)
        model_path.exists()
    }

    /// Downloads a model if it's not already cached.
    ///
    /// Uses `hf-hub` to download from Hugging Face.
    /// Defaults to `prajjwal1/bert-tiny` if model_name is generic.
    #[instrument(skip(self))]
    pub async fn ensure_model(&self, model_name: &str) -> Result<PathBuf> {
        self.init().await?;

        // Sanitize model name for local storage
        let safe_name = model_name.replace('/', "_");
        let model_path = self.models_dir.join(&safe_name);

        if self.is_cached(model_name) {
            info!("Model '{}' found in cache at {:?}", model_name, model_path);
            return Ok(model_path);
        }

        info!("Model '{}' not found. Starting download...", model_name);

        #[cfg(feature = "ai")]
        {
            self.download_from_hf(model_name, &model_path).await?;
        }

        #[cfg(not(feature = "ai"))]
        {
            warn!("AI feature not enabled. Using mock download.");
            self.mock_download(model_name, &model_path).await?;
        }

        info!(
            "Model '{}' downloaded successfully to {:?}",
            model_name, model_path
        );
        Ok(model_path)
    }

    /// Downloads model files from Hugging Face Hub.
    #[cfg(feature = "ai")]
    async fn download_from_hf(&self, model_id: &str, destination: &Path) -> Result<()> {
        let api = Api::new().context("Failed to create Hugging Face API client")?;
        let repo = api.repo(Repo::new(model_id.to_string(), RepoType::Model));

        info!("Fetching model files for {}...", model_id);

        // Files to download
        let files = vec!["config.json", "tokenizer.json", "model.safetensors"];

        fs::create_dir_all(destination).await?;

        for file in files {
            info!("Downloading {}...", file);
            let source_path = repo
                .get(file)
                .await
                .context(format!("Failed to download {}", file))?;

            // hf-hub stores files in its own cache structure.
            // We can either link to them or copy them.
            // For this project's explicit model management, we'll copy them to our managed dir.
            let dest_file = destination.join(file);
            fs::copy(&source_path, &dest_file).await?;
        }

        Ok(())
    }

    /// Returns the status of a specific model.
    pub async fn get_model_status(&self, model_name: &str) -> Result<ModelStatus> {
        let safe_name = model_name.replace('/', "_");
        let model_path = self.models_dir.join(&safe_name);
        let exists = self.is_cached(model_name);

        let size_mb = if exists {
            let mut total_size = 0;
            if let Ok(mut entries) = fs::read_dir(&model_path).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    if let Ok(metadata) = entry.metadata().await {
                        total_size += metadata.len();
                    }
                }
            }
            Some(total_size as f64 / 1_048_576.0)
        } else {
            None
        };

        Ok(ModelStatus {
            name: model_name.to_string(),
            size_mb,
            loaded: false, // We don't track runtime loading state in this simple manager yet
            path: if exists {
                Some(model_path.to_string_lossy().to_string())
            } else {
                None
            },
        })
    }

    /// Simulates downloading a model for environments without AI features or for testing.
    #[cfg(not(feature = "ai"))]
    async fn mock_download(&self, _model_name: &str, path: &PathBuf) -> Result<()> {
        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        fs::create_dir_all(path).await?;

        // Create dummy files
        fs::write(path.join("config.json"), b"{}").await?;
        fs::write(path.join("tokenizer.json"), b"{}").await?;
        fs::write(path.join("model.safetensors"), b"dummy model content")
            .await
            .context("Failed to write dummy model file")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_ensure_model_downloads_if_missing() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());
        let model_name = "test-model-bin"; // Avoid / in mock test

        // First call: should trigger "download"
        let path = manager.ensure_model(model_name).await.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("models"));

        // Verify status
        let status = manager.get_model_status(model_name).await.unwrap();
        assert_eq!(status.name, model_name);
        assert!(status.size_mb.is_some());
        assert!(status.path.is_some());
    }

    #[tokio::test]
    async fn test_is_cached() {
        let temp_dir = TempDir::new().unwrap();
        let manager = ModelManager::new(temp_dir.path());
        let model_name = "cached-model-bin";

        assert!(!manager.is_cached(model_name));

        manager.ensure_model(model_name).await.unwrap();

        assert!(manager.is_cached(model_name));
    }
}
