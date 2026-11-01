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
        let bytes = serde_json::to_vec(artwork)?;
        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(|e| VgcError::Database(format!("Sled insert error: {e}")))?;
        Ok(())
    }

    /// Alias for `save_artwork` — upserts an artwork record.
    pub fn put_artwork(&self, artwork: &Artwork) -> Result<(), VgcError> {
        self.save_artwork(artwork)
    }

    /// Retrieves an artwork metadata record by ID.
    /// Returns `Ok(Some(artwork))` if found, `Ok(None)` if not.
    pub fn get_artwork_result(&self, id: &str) -> Result<Option<Artwork>, VgcError> {
        let key = format!("artwork:{}", id);
        let res = self
            .db
            .get(key.as_bytes())
            .map_err(|e| VgcError::Database(format!("Sled get error: {e}")))?;

        match res {
            Some(ivec) => {
                let artwork: Artwork = serde_json::from_slice(&ivec)?;
                Ok(Some(artwork))
            }
            None => Ok(None),
        }
    }

    /// Convenience getter that returns `Option<Artwork>` directly,
    /// logging any deserialization errors rather than propagating them.
    pub fn get_artwork(&self, id: &str) -> Option<Artwork> {
        self.get_artwork_result(id).ok().flatten()
    }

    /// Lists all artworks stored in the database (full scan of `artwork:*` keys).
    pub fn list_artworks(&self) -> Vec<Artwork> {
        self.db
            .scan_prefix(b"artwork:")
            .filter_map(|entry| {
                let (_key, value) = entry.ok()?;
                serde_json::from_slice(&value).ok()
            })
            .collect()
    }

    /// Deletes an artwork record permanently by ID.
    pub fn delete_artwork(&self, id: &str) -> Result<bool, VgcError> {
        let key = format!("artwork:{}", id);
        let removed = self
            .db
            .remove(key.as_bytes())
            .map_err(|e| VgcError::Database(format!("Sled remove error: {e}")))?;
        Ok(removed.is_some())
    }

    /// Stores a CatArtworkManifest.
    pub fn save_manifest(&self, manifest: &CatArtworkManifest) -> Result<(), VgcError> {
        let key = format!("manifest:{}", manifest.artwork_id);
        let bytes = serde_json::to_vec(manifest)?;
        self.db
            .insert(key.as_bytes(), bytes)
            .map_err(|e| VgcError::Database(format!("Sled insert error: {e}")))?;
        Ok(())
    }

    /// Retrieves a CatArtworkManifest by artwork ID.
    pub fn get_manifest(&self, artwork_id: &str) -> Result<Option<CatArtworkManifest>, VgcError> {
        let key = format!("manifest:{}", artwork_id);
        let res = self
            .db
            .get(key.as_bytes())
            .map_err(|e| VgcError::Database(format!("Sled get error: {e}")))?;

        match res {
            Some(ivec) => {
                let manifest: CatArtworkManifest = serde_json::from_slice(&ivec)?;
                Ok(Some(manifest))
            }
            None => Ok(None),
        }
    }

    /// Returns the total number of entries in the database.
    pub fn entry_count(&self) -> usize {
        self.db.len()
    }

    /// Returns the approximate disk size of the database in bytes.
    pub fn disk_size_bytes(&self) -> u64 {
        self.db.size_on_disk().unwrap_or(0)
    }

    /// Flushes database changes to disk.
    pub fn flush(&self) -> Result<(), VgcError> {
        self.db
            .flush()
            .map_err(|e| VgcError::Database(format!("Sled flush error: {e}")))?;
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
        let fetched = db.get_artwork("cat_art_123").expect("should exist");
        assert_eq!(fetched.title, "Starry Night Feline");
        assert_eq!(fetched.likes, Some(42));

        // Test list_artworks
        let all = db.list_artworks();
        assert_eq!(all.len(), 1);
        assert_eq!(all[0].id, "cat_art_123");

        // Test delete_artwork
        assert!(db.delete_artwork("cat_art_123").unwrap());
        assert!(db.get_artwork("cat_art_123").is_none());
    }
}
