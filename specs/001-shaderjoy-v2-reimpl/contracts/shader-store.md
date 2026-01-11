# Contract: Shader Store Trait

**Module**: `shaderjoy-core/src/storage/trait.rs`

## Purpose

Abstract interface for shader persistence enabling filesystem and database backends.

---

## Trait Definition

```rust
#[async_trait]
pub trait ShaderStore: Send + Sync {
    /// Save a specimen to storage.
    async fn save(&self, specimen: &Specimen) -> Result<(), StorageError>;
    
    /// Load a specimen by ID.
    async fn load(&self, id: Uuid) -> Result<Option<Specimen>, StorageError>;
    
    /// List recent specimens, ordered by creation time (newest first).
    async fn list_recent(&self, limit: usize) -> Result<Vec<Specimen>, StorageError>;
    
    /// Load the complete lineage (all ancestors) of a specimen.
    /// Returns specimens ordered oldest-to-newest.
    async fn load_lineage(&self, id: Uuid) -> Result<Vec<Specimen>, StorageError>;
    
    /// Save a complete session with naming.
    /// 
    /// # Arguments
    /// * `name` - User-provided session name (filesystem-safe)
    /// * `specimens` - Intermediate evolution steps
    /// * `final_specimen` - The final selected shader
    /// 
    /// # Returns
    /// Path to saved session directory/location.
    async fn save_session(
        &self,
        name: &str,
        specimens: &[Specimen],
        final_specimen: &Specimen,
    ) -> Result<PathBuf, StorageError>;
    
    /// Check if a session name already exists.
    async fn session_exists(&self, name: &str) -> Result<bool, StorageError>;
}
```

---

## Error Types

```rust
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Database error: {0}")]
    Database(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Session already exists: {0}")]
    SessionExists(String),
    
    #[error("Invalid session name: {0}")]
    InvalidName(String),
}
```

---

## Implementations

### FilesystemStore

```rust
pub struct FilesystemStore {
    root: PathBuf,
}

impl FilesystemStore {
    pub fn new(root: impl Into<PathBuf>) -> Result<Self, StorageError>;
}
```

**Directory structure:**
```
{root}/{session-name}/
├── step1.wgsl
├── step1.json
├── final.wgsl
└── final.json
```

### DatabaseStore

```rust
pub struct DatabaseStore {
    db: DatabaseConnection,
}

impl DatabaseStore {
    pub async fn new(database_url: &str) -> Result<Self, StorageError>;
}
```

---

## Factory Function

```rust
pub async fn create_shader_store(config: &StorageConfig) -> Result<Arc<dyn ShaderStore>, StorageError> {
    match config.backend {
        StorageBackend::Filesystem => {
            Ok(Arc::new(FilesystemStore::new(&config.path)?))
        }
        StorageBackend::Database => {
            let url = config.database_url.as_ref()
                .ok_or_else(|| StorageError::Database("Missing database_url".into()))?;
            Ok(Arc::new(DatabaseStore::new(url).await?))
        }
    }
}
```

---

## Usage Example

```rust
let store = create_shader_store(&config).await?;

// Save a session
store.save_session(
    "my-plasma-shader",
    &evolution_steps,
    &final_specimen,
).await?;

// Load lineage for display
let history = store.load_lineage(specimen.id).await?;
```
