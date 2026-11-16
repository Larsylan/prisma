use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::archive::LibraryArchive;
use super::{SourceCache, TrophyLibrary};
use crate::domain::Timestamp;

#[derive(Clone, Debug, Default)]
pub struct StoredLibrary {
    pub library: TrophyLibrary,
    pub cache: SourceCache,
}

#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {
    #[error("lecture ou écriture impossible : {0}")]
    Io(#[from] io::Error),
    #[error("bibliothèque illisible : {0}")]
    Format(#[from] serde_json::Error),
}

pub struct LibraryRepository {
    directory: PathBuf,
}

const FILE_NAME: &str = "library.json";

impl LibraryRepository {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        LibraryRepository { directory: directory.into() }
    }

    pub fn directory(&self) -> &Path {
        &self.directory
    }

    /// Retourne une bibliothèque vide si le fichier n'existe pas encore.
    pub fn load(&self) -> Result<StoredLibrary, RepositoryError> {
        match fs::read(self.file()) {
            Ok(data) => Ok(serde_json::from_slice::<LibraryArchive>(&data)?.into_stored()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(StoredLibrary::default()),
            Err(error) => Err(error.into()),
        }
    }

    /// L'écriture passe par un fichier temporaire : une coupure ne laisse jamais un fichier tronqué.
    pub fn save(&self, stored: &StoredLibrary) -> Result<(), RepositoryError> {
        fs::create_dir_all(&self.directory)?;
        let temporary = self.directory.join(format!("{FILE_NAME}.tmp"));
        fs::write(&temporary, serde_json::to_vec(&LibraryArchive::new(stored))?)?;
        fs::rename(temporary, self.file())?;
        Ok(())
    }

    /// Met de côté un fichier illisible pour qu'il ne soit pas écrasé par une bibliothèque vide.
    pub fn quarantine_unreadable_file(&self, now: Timestamp) -> Result<PathBuf, RepositoryError> {
        let destination = self.directory.join(format!("library.unreadable-{}.json", now.timestamp()));
        fs::rename(self.file(), &destination)?;
        Ok(destination)
    }

    fn file(&self) -> PathBuf {
        self.directory.join(FILE_NAME)
    }
}
