//! Local file storage engine for raw artwork blobs and .catpack bundles.

use std::path::{Path, PathBuf};
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use vangoghcat_core::VgcError;

pub struct FileStorageEngine {
    base_dir: PathBuf,
}

impl FileStorageEngine {
    pub async fn new<P: AsRef<Path>>(base_dir: P) -> Result<Self, VgcError> {
        let path = base_dir.as_ref().to_path_buf();
        fs::create_dir_all(&path)
            .await
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to create storage dir: {e}")))?;
        Ok(Self { base_dir: path })
    }

    pub fn blob_path(&self, id: &str) -> PathBuf {
        self.base_dir.join(format!("{}.blob", id))
    }

    /// Asynchronously writes bytes to disk.
    pub async fn write_blob(&self, id: &str, data: &[u8]) -> Result<PathBuf, VgcError> {
        let target = self.blob_path(id);
        let mut file = File::create(&target)
            .await
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to create blob file: {e}")))?;
        file.write_all(data)
            .await
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to write blob data: {e}")))?;
        file.flush()
            .await
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to flush blob: {e}")))?;
        Ok(target)
    }

    /// Asynchronously reads full blob bytes.
    pub async fn read_blob(&self, id: &str) -> Result<Vec<u8>, VgcError> {
        let target = self.blob_path(id);
        fs::read(&target)
            .await
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to read blob: {e}")))
    }

    /// Checks if a blob exists on disk.
    pub fn has_blob(&self, id: &str) -> bool {
        self.blob_path(id).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_file_storage_roundtrip() {
        let tmp = std::env::temp_dir().join(format!("vgc_test_{}", std::process::id()));
        let engine = FileStorageEngine::new(&tmp).await.expect("engine");

        let data = b"Van Gogh Cat high-resolution masterpiece canvas data";
        let blob_id = "test_blob_999";

        engine.write_blob(blob_id, data).await.expect("write");
        assert!(engine.has_blob(blob_id));

        let loaded = engine.read_blob(blob_id).await.expect("read");
        assert_eq!(loaded, data);
        let _ = tokio::fs::remove_dir_all(&tmp).await;
    }
}
