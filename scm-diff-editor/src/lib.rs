//! An interactive difftool for use in VCS programs like
//! [Jujutsu](https://github.com/martinvonz/jj) or Git.
#![warn(missing_docs)]
#![warn(
    clippy::all,
    clippy::as_conversions,
    clippy::clone_on_ref_ptr,
    clippy::dbg_macro
)]
#![allow(clippy::too_many_arguments)]

mod render;
pub mod testing;

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::error;
use std::fmt::Display;
use std::io;
use std::path::{Path, PathBuf, StripPrefixError};

use clap::Parser;
use scm_record::{File, FileMode, RecordError, RecordState, SelectedContents};

/// Render a partial commit selector for use as a difftool or mergetool.
///
/// This can be used to interactively select changes to include as part of a
/// commit, to resolve merge conflicts, or to simply display a diff in a
/// readable way.
#[derive(Debug, Parser)]
pub struct Opts {
    /// Instead of comparing two files, compare two directories recursively.
    #[clap(short = 'd', long = "dir-diff")]
    pub dir_diff: bool,

    /// The left-hand file to compare (or directory if `--dir-diff` is passed).
    pub left: PathBuf,

    /// The right-hand file to compare (or directory if `--dir-diff` is passed).
    pub right: PathBuf,

    /// Disable all editing controls and do not write the selected commit
    /// contents to disk.
    #[clap(long = "read-only")]
    pub read_only: bool,

    /// Show what would have been written to disk as part of the commit
    /// selection, but do not actually write it.
    #[clap(short = 'N', long = "dry-run")]
    pub dry_run: bool,

    /// Render the interface as a mergetool instead of a difftool and use this
    /// file as the base of a three-way diff as part of resolving merge
    /// conflicts.
    #[clap(
        short = 'b',
        long = "base",
        requires("output"),
        conflicts_with("dir_diff")
    )]
    pub base: Option<PathBuf>,

    /// Write the resolved merge conflicts to this file.
    #[clap(short = 'o', long = "output", conflicts_with("dir_diff"))]
    pub output: Option<PathBuf>,
}

#[derive(Debug)]
#[allow(missing_docs)]
pub enum Error {
    Cancelled,
    DryRun,
    WalkDir {
        source: walkdir::Error,
    },
    StripPrefix {
        root: PathBuf,
        path: PathBuf,
        source: StripPrefixError,
    },
    ReadFile {
        path: PathBuf,
        source: io::Error,
    },
    RemoveFile {
        path: PathBuf,
        source: io::Error,
    },
    CopyFile {
        old_path: PathBuf,
        new_path: PathBuf,
        source: io::Error,
    },
    CreateDirAll {
        path: PathBuf,
        source: io::Error,
    },
    WriteFile {
        path: PathBuf,
        source: io::Error,
    },
    MissingMergeFile {
        path: PathBuf,
    },
    BinaryMergeFile {
        path: PathBuf,
    },
    Record {
        source: RecordError,
    },
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Cancelled => {
                write!(f, "aborted by user")
            }
            Error::DryRun => {
                write!(f, "dry run, not writing any files")
            }
            Error::WalkDir { source } => {
                write!(f, "walking directory: {source}")
            }
            Error::StripPrefix { root, path, source } => {
                write!(
                    f,
                    "stripping directory prefix {} from {}: {source}",
                    root.display(),
                    path.display()
                )
            }
            Error::ReadFile { path, source } => {
                write!(f, "reading file {}: {source}", path.display())
            }
            Error::RemoveFile { path, source } => {
                write!(f, "removing file {}: {source}", path.display())
            }
            Error::CopyFile {
                old_path,
                new_path,
                source,
            } => {
                write!(
                    f,
                    "copying file {} to {}: {source}",
                    old_path.display(),
                    new_path.display()
                )
            }
            Error::CreateDirAll { path, source } => {
                write!(f, "creating directory {}: {source}", path.display())
            }
            Error::WriteFile { path, source } => {
                write!(f, "writing file {}: {source}", path.display())
            }
            Error::MissingMergeFile { path } => {
                write!(f, "file did not exist: {}", path.display())
            }
            Error::BinaryMergeFile { path } => {
                write!(f, "file was not text: {}", path.display())
            }
            Error::Record { source } => {
                write!(f, "recording changes: {source}")
            }
        }
    }
}

/// Result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Abstraction over the filesystem.
pub trait Filesystem {
    /// Find the set of files that appear in either `left` or `right`.
    fn read_dir_diff_paths(&self, left: &Path, right: &Path) -> Result<BTreeSet<PathBuf>>;

    /// Read the [`FileInfo`] for the provided `path`.
    fn read_file_info(&self, path: &Path) -> Result<FileInfo>;

    /// Write new file contents to `path`.
    fn write_file(&mut self, path: &Path, contents: &str) -> Result<()>;

    /// Copy the file at `old_path` to `new_path`. (This can be more efficient
    /// than reading and writing the entire contents, particularly for large
    /// binary files.)
    fn copy_file(&mut self, old_path: &Path, new_path: &Path) -> Result<()>;

    /// Delete the file at `path`.
    fn remove_file(&mut self, path: &Path) -> Result<()>;

    /// Create the directory `path` and any parent directories as necessary.
    fn create_dir_all(&mut self, path: &Path) -> Result<()>;
}

/// Information about a file that was read from disk. Note that the file may not have existed, in
/// which case its contents will be marked as absent.
#[derive(Clone, Debug)]
pub struct FileInfo {
    /// The file mode (see [`scm_record::FileMode`]).
    pub file_mode: FileMode,

    /// The file contents.
    pub contents: FileContents,
}

/// Representation of a file's contents.
#[derive(Clone, Debug)]
pub enum FileContents {
    /// There is no file. (This is different from the file being present but empty.)
    Absent,

    /// The file is a text file with the given contents.
    Text {
        /// The contents of the file.
        contents: String,

        /// The hash of [`contents`].
        hash: String,

        /// The size of [`contents`], in bytes.
        num_bytes: u64,
    },

    /// The file is a binary file (not able to be displayed directly in the UI).
    Binary {
        /// The hash of the file's contents.
        hash: String,

        /// The size of the file's contents, in bytes.
        num_bytes: u64,
    },
}

/// Information about the files to display/diff in the UI.
#[derive(Debug)]
pub struct DiffContext {
    /// The files to diff.
    /// - When diffing a single file, this will have only one entry.
    /// - When diffing a directory, this may have many entries (one for each pair of files).
    pub files: Vec<File<'static>>,

    /// When writing results to the filesystem, this path should be prepended to
    /// each `File`'s path. It may be empty (indicating to overwrite the file
    /// in-place).
    pub write_root: PathBuf,
}

/// Process the command-line options to find the files to diff.
pub fn process_opts(filesystem: &dyn Filesystem, opts: &Opts) -> Result<DiffContext> {
    let result = match opts {
        Opts {
            dir_diff: false,
            left,
            right,
            base: None,
            output: _,
            read_only: _,
            dry_run: _,
        } => {
            let files = vec![render::create_file(
                filesystem,
                left.clone(),
                left.clone(),
                right.clone(),
                right.clone(),
            )?];
            DiffContext {
                files,
                write_root: PathBuf::new(),
            }
        }

        Opts {
            dir_diff: true,
            left,
            right,
            base: None,
            output: _,
            read_only: _,
            dry_run: _,
        } => {
            let display_paths = filesystem.read_dir_diff_paths(left, right)?;
            let mut files = Vec::new();
            for display_path in display_paths {
                files.push(render::create_file(
                    filesystem,
                    left.join(&display_path),
                    display_path.clone(),
                    right.join(&display_path),
                    display_path.clone(),
                )?);
            }
            DiffContext {
                files,
                write_root: right.clone(),
            }
        }

        Opts {
            dir_diff: false,
            left,
            right,
            base: Some(base),
            output: Some(output),
            read_only: _,
            dry_run: _,
        } => {
            let files = vec![render::create_merge_file(
                filesystem,
                base.clone(),
                left.clone(),
                right.clone(),
                output.clone(),
            )?];
            DiffContext {
                files,
                write_root: PathBuf::new(),
            }
        }

        Opts {
            dir_diff: false,
            left: _,
            right: _,
            base: Some(_),
            output: None,
            read_only: _,
            dry_run: _,
        } => {
            unreachable!("--output is required when --base is provided");
        }

        Opts {
            dir_diff: true,
            left: _,
            right: _,
            base: Some(_),
            output: _,
            read_only: _,
            dry_run: _,
        } => {
            unimplemented!("--base cannot be used with --dir-diff");
        }
    };
    Ok(result)
}

/// After the user has selected changes in the provided [`RecordState`], write
/// the results to the provided [`Filesystem`].
pub fn apply_changes(
    filesystem: &mut dyn Filesystem,
    write_root: &Path,
    state: RecordState,
) -> Result<()> {
    let RecordState {
        is_read_only,
        commits: _,
        files,
    } = state;
    if is_read_only {
        return Ok(());
    }
    for file in files {
        let file_path = write_root.join(file.path.clone());
        let (selected_contents, _unselected_contents) = file.get_selected_contents();
        match selected_contents {
            SelectedContents::Absent => {
                filesystem.remove_file(&file_path)?;
            }
            SelectedContents::Unchanged => {
                // Do nothing.
            }
            SelectedContents::Binary {
                old_description: _,
                new_description: _,
            } => {
                let new_path = file_path;
                let old_path = match &file.old_path {
                    Some(old_path) => old_path.clone(),
                    None => Cow::Borrowed(new_path.as_path()),
                };
                filesystem.copy_file(&old_path, &new_path)?;
            }
            SelectedContents::Present { contents } => {
                if let Some(parent_dir) = file_path.parent() {
                    filesystem.create_dir_all(parent_dir)?;
                }
                filesystem.write_file(&file_path, &contents)?;
            }
        }
    }
    Ok(())
}
