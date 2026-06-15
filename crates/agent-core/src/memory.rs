//! Memory management for agent message history.

use crate::llm::ChatMessage;
use std::fs;
use std::path::{Path, PathBuf};

/// Trait managing message history for the agent.
pub trait Memory: Send + Sync {
    /// Append a message to the history.
    fn add_message(&mut self, message: ChatMessage);

    /// Retrieve all messages currently in memory.
    fn messages(&self) -> &[ChatMessage];

    /// Clear all messages in the history.
    fn clear(&mut self);
}

/// Simple in-memory message history store.
#[derive(Debug, Clone, Default)]
pub struct SimpleMemory {
    messages: Vec<ChatMessage>,
}

impl SimpleMemory {
    /// Create a new SimpleMemory store.
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
        }
    }
}

impl Memory for SimpleMemory {
    fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
    }

    fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    fn clear(&mut self) {
        self.messages.clear();
    }
}

/// Error type for FileMemory operations.
#[derive(Debug, thiserror::Error)]
pub enum FileMemoryError {
    /// An I/O error occurred reading or writing the history file.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// The history file contained invalid JSON.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

/// JSON-backed persistent memory. Loads existing history on creation
/// and writes back to disk after every new message.
#[derive(Debug, Clone)]
pub struct FileMemory {
    path: PathBuf,
    messages: Vec<ChatMessage>,
}

impl FileMemory {
    /// Load (or create) a history file at `path`.
    ///
    /// If the file does not exist it is created empty on the first write.
    /// If it exists but cannot be parsed the error is returned immediately so
    /// the caller can decide whether to proceed with an empty history or abort.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, FileMemoryError> {
        let path = path.as_ref().to_path_buf();
        let messages = if path.exists() {
            let bytes = fs::read(&path)?;
            serde_json::from_slice(&bytes)?
        } else {
            Vec::new()
        };
        Ok(Self { path, messages })
    }

    /// Flush the current message list to the history file.
    fn flush(&self) -> Result<(), FileMemoryError> {
        // Create parent directories if they don't exist
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        let bytes = serde_json::to_vec_pretty(&self.messages)?;
        fs::write(&self.path, bytes)?;
        Ok(())
    }

    /// Return the path this store is backed by.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Memory for FileMemory {
    fn add_message(&mut self, message: ChatMessage) {
        self.messages.push(message);
        if let Err(e) = self.flush() {
            tracing::warn!(
                "Failed to persist message to {}: {}",
                self.path.display(),
                e
            );
        }
    }

    fn messages(&self) -> &[ChatMessage] {
        &self.messages
    }

    fn clear(&mut self) {
        self.messages.clear();
        if let Err(e) = self.flush() {
            tracing::warn!(
                "Failed to clear history file {}: {}",
                self.path.display(),
                e
            );
        }
    }
}
