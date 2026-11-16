mod archive;
mod grouper;
mod repository;
mod source_cache;
mod title;
mod trophy_library;

pub use grouper::{GameGrouper, GroupingMode};
pub use repository::{LibraryRepository, RepositoryError, StoredLibrary};
pub use source_cache::SourceCache;
pub use trophy_library::{Change, TrophyLibrary};

#[cfg(test)]
mod tests;
