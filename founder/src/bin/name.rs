extern crate arguments;
extern crate founder;

use std::io::Result;
use std::path::{Path, PathBuf};

fn main() {
    let arguments = arguments::parse(std::env::args()).unwrap();
    let path: PathBuf = match arguments.get::<String>("path") {
        Some(path) => path.into(),
        _ => {
            eprintln!("Error: --path should be given.");
            return;
        }
    };
    let output: Option<PathBuf> = arguments
        .get::<String>("output")
        .map(|output| output.into());
    founder::scanning::scan_summarize(
        &path,
        filter,
        process,
        output,
        arguments.get::<usize>("workers").unwrap_or(1),
        &arguments.get_all::<String>("ignore").unwrap_or(vec![]),
    );
}

fn filter(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .map(|extension| ["otf", "ttf"].contains(&extension))
        .unwrap_or(false)
}

fn process(path: &Path, output: Option<PathBuf>) -> Result<Option<()>> {
    use std::fs::File;
    use std::io::Write;

    match subprocess(path) {
        Ok(result) => {
            match output {
                Some(output) => {
                    let output = output.join(path.file_stem().unwrap()).with_extension("txt");
                    let mut file = File::create(output)?;
                    write!(file, "{result}")?;
                }
                _ => println!("{result}"),
            }
            eprintln!("[success] {path:?}");
            Ok(Some(()))
        }
        Err(error) => {
            eprintln!("[failure] {path:?} ({error:?})");
            Err(error)
        }
    }
}

fn subprocess(path: &Path) -> Result<String> {
    use font::File;
    use std::fmt::Write;

    let File { mut fonts } = File::open(path)?;
    let mut string = String::new();
    for ((name_id, language_tag), value) in fonts[0].names()?.iter() {
        let name_id = format!("{name_id:?}");
        let language_tag = language_tag.as_deref().unwrap_or("--");
        let value = truncate(value.as_deref().unwrap_or("--"));
        writeln!(string, "{name_id: <25} {language_tag: <5} {value}").unwrap();
    }
    Ok(string)
}

fn truncate(string: &str) -> String {
    const MAX: usize = 50;
    let count = string.chars().count();
    let mut string = match string.char_indices().nth(MAX) {
        None => string.to_owned(),
        Some((index, _)) => string[..index].to_owned(),
    };
    if count > MAX {
        string.push('…');
    }
    string
}
