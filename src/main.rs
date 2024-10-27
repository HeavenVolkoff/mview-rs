fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let file = std::fs::File::open(filename)?;
    let mview_file = mview::MViewFile::new(file);

    for entry in mview_file {
        match entry {
            Ok(entry) => {
                println!("Name: {}, MIME Type: {}", entry.name, entry.mime_type);
                if std::path::Path::new(&entry.name).file_stem()
                    == Some(std::ffi::OsStr::new("thumbnail"))
                {
                    std::fs::write(&entry.name, &entry.data)?;
                }
            }
            Err(e) => eprintln!("Error: {e}"),
        }
    }

    Ok(())
}
