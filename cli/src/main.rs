//! mview-extract is a command-line utility for extracting the contents of Marmoset Viewer archive packages.

use std::{io, path::PathBuf};

use clap::{command, ArgAction, Parser};
use mview::extract_mview;

#[derive(Parser, Debug)]
#[command(about = "Extract contents from .mview files", version, author)]
struct Args {
    /// Path to the .mview file to be extracted
    #[arg(required = true)]
    filename: PathBuf,

    /// Output directory for extracted files (defaults to current directory)
    #[arg(short, long, value_name = "DIR")]
    output_dir: Option<PathBuf>,

    /// Do not create a subdirectory for extracted files
    #[arg(long = "no-create-subdir", action = ArgAction::SetFalse)]
    create_subdir: bool,
}

fn parse_args() -> io::Result<(PathBuf, PathBuf, bool)> {
    let args = Args::parse();

    let output_dir = match args.output_dir {
        Some(dir) => dir,
        None => std::env::current_dir()?,
    };

    Ok((args.filename, output_dir, args.create_subdir))
}

fn main() -> io::Result<()> {
    let (filename, output_dir, create_subdir) = parse_args()?;
    extract_mview(&filename, &output_dir, create_subdir)
}
