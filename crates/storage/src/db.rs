//! Sled KV database engine for manifests, peer contracts, and artwork metadata.

use sled::Db;
use std::path::Path;
use vangoghcat_core::{types::*, VgcError};

pub struct StorageDb {
    db: Db,
}

impl StorageDb {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, VgcError> {
        let db = sled::open(path)
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to open sled db: {e}")))?;
        Ok(Self { db })
    }

    pub fn open_temporary() -> Result<Self, VgcError> {
        let db = sled::Config::new()
            .temporary(true)
            .open()
            .map_err(|e| VgcError::InvalidNodeId(format!("Failed to open temp db: {e}")))?;
        Ok(Self { db })
    }

    /// Stores an artwork metadata record.
    pub fn save_artwork(&self, artwork: &Artwork) -> Result<(), VgcError> {
        let key = format!("artwork:{}", artwork.id);
        let bytes = serde_json::to_vec(artwork)
            .map_err(|e| VgcError::InvalidNodeId(format!("JSON serialization error: {e}")))?;
        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(|e| VgcError::InvalidNodeId(format!("Sled insert error: {e}")))?;
        Ok(())
    }

    /// Retrieves an artwork metadata record by ID.
    pub fn get_artwork(&self, id: &str) -> Result<Option<Artwork>, VgcError> {
        let key = format!("artwork:{}", id);
        let res = self
            .db
            .get(key.as_bytes())
            .map_err(|e| VgcError::InvalidNodeId(format!("Sled get error: {e}")))?;

        match res {
            Some(ivec) => {
                let artwork: Artwork = serde_json::from_slice(&ivec).map_err(|e| {
                    VgcError::InvalidNodeId(format!("JSON deserialization error: {e}"))
                })?;
                Ok(Some(artwork))
            }
            None => Ok(None),
        }
    }

    /// Stores a CatArtworkManifest.
    pub fn save_manifest(&self, manifest: &CatArtworkManifest) -> Result<(), VgcError> {
        let key = format!("manifest:{}", manifest.artwork_id);
        let bytes = serde_json::to_vec(manifest)
            .map_err(|e| VgcError::InvalidNodeId(format!("JSON serialization error: {e}")))?;
        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(|e| VgcError::InvalidNodeId(format!("Sled insert error: {e}")))?;
        Ok(())
    }

    /// Retrieves a CatArtworkManifest by artwork ID.
    pub fn get_manifest(&self, artwork_id: &str) -> Result<Option<CatArtworkManifest>, VgcError> {
        let key = format!("manifest:{}", artwork_id);
        let res = self
            .db
            .get(key.as_bytes())
            .map_err(|e| VgcError::InvalidNodeId(format!("Sled get error: {e}")))?;

        match res {
            Some(ivec) => {
                let manifest: CatArtworkManifest = serde_json::from_slice(&ivec).map_err(|e| {
                    VgcError::InvalidNodeId(format!("JSON deserialization error: {e}"))
                })?;
                Ok(Some(manifest))
            }
            None => Ok(None),
        }
    }

    /// Flushes database changes to disk.
    pub fn flush(&self) -> Result<(), VgcError> {
        self.db
            .flush()
            .map_err(|e| VgcError::InvalidNodeId(format!("Sled flush error: {e}")))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_db_artwork_crud() {
        let db = StorageDb::open_temporary().expect("temp db should open");
        let artwork = Artwork {
            id: "cat_art_123".into(),
            title: "Starry Night Feline".into(),
            title_en: Some("Starry Night Feline".into()),
            image_url: "blob://canvas".into(),
            prompt: "Vincent van Gogh style cat with thick impasto brushstrokes".into(),
            style: "Post-Impressionism".into(),
            year: "1889".into(),
            description: "A majestic ginger cat under the swirling stars of Saint-Rémy".into(),
            palette: vec![ColorSwatch {
                hex: "#1B2F5C".into(),
                name: "Prussian Blue".into(),
            }],
            brushstroke_note: "Heavy swirling impasto technique".into(),
            aspect_ratio: AspectRatio::Square,
            created_at: 1730000000,
            is_user_created: Some(true),
            model_used: Some("WebGPU-Diffusion-v2".into()),
            is_svg_art: Some(false),
            likes: Some(42),
            likes_count: Some(42),
            style_preset: Some("Starry Night".into()),
            prompt_used: None,
            author_id: Some("catnode1alice".into()),
            author_name: Some("Vincent van Cat".into()),
            author_signature: None,
            origin_peer_id: None,
            is_backed_up_locally: Some(true),
            source: Some(ArtworkSource::MyLocal),
            is_condemned: Some(false),
            condemned_reason: None,
            cat_profile_id: None,
            zkp_commitment: None,
            zkp_fingerprint: None,
            zkp_revealed: None,
        };

        db.save_artwork(&artwork).expect("should save");
        let fetched = db
            .get_artwork("cat_art_123")
            .expect("should query")
            .expect("should exist");
        assert_eq!(fetched.title, "Starry Night Feline");
        assert_eq!(fetched.likes, Some(42));
    }
}
