use std::{env, io, path::Path};

use mview::extract_mview;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <filename> [output_directory]", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let output_dir = if args.len() == 3 {
        Path::new(&args[2])
    } else {
        &env::current_dir()?
    };

    extract_mview(Path::new(filename), output_dir, true)
}
