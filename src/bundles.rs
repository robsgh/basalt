use anyhow::{Context, Result, bail};
use std::{
    borrow::Cow,
    fs,
    path::{Path, PathBuf},
};

use log::{debug, error};

/// The name of a note that fails to load from disk
pub const NOTE_BAD_NAME: &'static str = "<<error>>";

/// A single markdown note that is cached into memory after the first load
#[derive(Debug, Default, Clone)]
pub struct Note {
    /// Path to the note on disk
    path: PathBuf,
    /// Loaded note content from disk
    content: Option<String>,
}

impl Note {
    /// Load the note from disk unconditionally
    ///
    /// This will reload the note from disk even if `self.content` is already `Some(_)`
    pub fn load(&mut self) -> Result<()> {
        let c = fs::read_to_string(&self.path).with_context(|| {
            format!(
                "failed to read note in bundle at {}",
                self.path.to_string_lossy()
            )
        })?;

        self.content = Some(c);
        Ok(())
    }

    /// Returns true when the note has been loaded from disk and `self.content` is populated
    pub fn is_loaded(&self) -> bool {
        self.content.is_some()
    }

    /// Get the content of this note as a [`&str`]
    pub fn get(&self) -> Option<&str> {
        self.content.as_deref()
    }

    /// Get the name of the note
    pub fn name(&self) -> Cow<'_, str> {
        match self.path.file_name() {
            Some(n) => n.to_string_lossy(),
            None => NOTE_BAD_NAME.into(),
        }
    }
}

/// A Bundle of notes
#[derive(Debug, Default, Clone)]
pub struct Bundle {
    /// Path to the bundle directory on disk
    path: PathBuf,
    /// [`Vec`] of [`Note`] structs that will cache on-disk contents after first load
    notes: Vec<Note>,
}

impl Bundle {
    /// Get the name of the Bundle (directory name)
    pub fn name(&self) -> String {
        if let Some(name) = self.path.file_name() {
            name.to_string_lossy().to_string()
        } else {
            "<<ERROR BUNDLE NAME>>".into()
        }
    }

    /// Get (and load, if necessary) a note at an index
    pub fn get_note(&mut self, idx: usize) -> Option<&Note> {
        {
            if let Some(n) = self.notes.get_mut(idx)
                && !n.is_loaded()
            {
                let res = n.load();
                if let Err(e) = res {
                    error!("failed to get note {}: {e:?}", n.name());
                }
            }
        }

        self.notes.get(idx)
    }

    /// Get the path to the bundle directory on disk
    pub fn get_path(&self) -> &Path {
        &self.path
    }

    /// Get the note names in this bundle
    pub fn get_note_names(&self) -> Vec<String> {
        self.notes
            .iter()
            .map(|note| note.name().to_string())
            .collect()
    }
}

impl IntoIterator for Bundle {
    type Item = Note;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.notes.into_iter()
    }
}

/// A convenience loader for Basalt [`Bundle`] structs
#[derive(Debug, Default, Clone)]
pub struct BundleLoader {
    /// Path to the bundle directory on disk
    path: PathBuf,
}

impl BundleLoader {
    /// Create a Basalt Bundle with a given path
    pub fn new(path: &Path) -> Self {
        let path = path.to_path_buf();

        Self { path }
    }

    /// Init the bundle at the given path
    pub fn init(self) -> Result<Bundle> {
        debug!(
            "Creating Bundle directory at {}",
            self.path.to_string_lossy()
        );

        fs::create_dir_all(&self.path).with_context(|| {
            format!(
                "failed to create basalt Bundle directory at {}",
                self.path.to_string_lossy()
            )
        })?;
        let notes = Vec::new();

        Ok(Bundle {
            path: self.path,
            notes,
        })
    }

    /// Load the Bundle from disk
    pub fn load(self) -> Result<Bundle> {
        debug!("loading Bundle from {}", self.path.to_string_lossy());

        let notes = fs::read_dir(&self.path)?
            .filter_map(|d| d.ok().map(|entry| entry.path()))
            .map(|path| Note {
                path,
                content: None,
            })
            .collect();

        Ok(Bundle {
            path: self.path,
            notes,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, fs::File};

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn handle_read_file() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("test.md");
        let test_file_content = "this is a test file";
        File::create(&file_path)?;
        fs::write(&file_path, test_file_content)?;

        let mut note = Note {
            path: file_path,
            ..Default::default()
        };
        assert!(!note.is_loaded());

        note.load()?;

        assert!(note.is_loaded());

        assert_eq!(note.get(), Some(test_file_content));

        Ok(())
    }

    #[test]
    fn bad_name_has_error_name() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("..");

        let note = Note {
            path: file_path,
            ..Default::default()
        };

        assert!(!note.is_loaded());
        assert_eq!(note.name(), NOTE_BAD_NAME);

        Ok(())
    }

    #[test]
    fn test_init_bundle_from_loader() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("init_test");

        let bundle_loader = BundleLoader::new(&path);

        assert!(fs::exists(&path).is_ok_and(|exists| !exists));

        let bundle = bundle_loader.init()?;
        assert!(fs::exists(&path).is_ok_and(|exists| exists));
        assert_eq!(bundle.path, path);
        assert_eq!(bundle.notes.len(), 0);

        Ok(())
    }

    #[test]
    fn test_load_bundle_from_loader() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join("init_test");
        fs::create_dir(&path)?;
        File::create_new(&path.join("file1.md"))?;
        File::create_new(&path.join("file2.md"))?;
        assert!(fs::exists(&path).is_ok_and(|exists| exists));

        let bundle = BundleLoader::new(&path).load()?;

        assert_eq!(bundle.path, path);
        assert_eq!(bundle.notes.len(), 2);
        assert_eq!(
            bundle.get_note_names().into_iter().collect::<HashSet<_>>(),
            HashSet::from([String::from("file1.md"), String::from("file2.md")])
        );

        Ok(())
    }
}
