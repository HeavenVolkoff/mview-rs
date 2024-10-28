//! [![](https://github.com/HeavenVolkoff/mview-rs/blob/main/.github/icon.svg)](https://github.com/HeavenVolkoff/mview-rs)
//!
//! mview is a library for library for parsing Marmoset Viewer archive packages

mod mview_entry;
mod mview_file;
mod utils;

use std::{
    fs::File,
    io::{self, Write},
    path::Path,
};

pub use mview_file::MViewFile;

/// Extracts the contents of an .mview file to the specified output directory.
///
/// # Arguments
///
/// * `file_path` - The path to the .mview file to extract.
/// * `output_dir` - The directory to extract the contents to.
/// * `create_subdir` - Whether to create a subdirectory for the extracted contents.
///
/// # Errors
///
/// This function will return an error if the file cannot be opened, read, or written to the output directory.
///
/// # Panics
///
/// This function will panic if the file stem cannot be determined from the file path.
pub fn extract_mview(file_path: &Path, output_dir: &Path, create_subdir: bool) -> io::Result<()> {
    let entries = MViewFile::new(File::open(file_path)?).collect::<io::Result<Vec<_>>>()?;
    let output_dir = if create_subdir {
        let file_stem = file_path
            .file_stem()
            .expect("Should work considering we opened the file")
            .to_string_lossy()
            .to_string();
        output_dir.join(file_stem)
    } else {
        output_dir.to_path_buf()
    };

    std::fs::create_dir_all(&output_dir)?;

    for entry in entries {
        let output_file = output_dir.join(entry.name);
        let mut output = File::create(output_file)?;
        output.write_all(&entry.data)?;
    }

    Ok(())
}
