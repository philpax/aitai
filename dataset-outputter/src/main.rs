use serde::{Deserialize, Serialize};
use std::{
    env,
    fmt::Write,
    fs::File,
    io::{BufRead, BufReader, Write as IoWrite},
    path::PathBuf,
};
use unicode_normalization::UnicodeNormalization;

fn main() -> anyhow::Result<()> {
    let input_path = match env::args().nth(1) {
        Some(path) => {
            let path = PathBuf::from(path);
            if !path.exists() {
                anyhow::bail!("Input file does not exist");
            }
            if path.extension().unwrap_or_default() != "ndjson" {
                anyhow::bail!("Input file must be an ndjson file");
            }
            path
        }
        None => {
            anyhow::bail!("No input file specified");
        }
    };

    let output_path = match env::args().nth(2) {
        Some(path) => PathBuf::from(path),
        None => {
            anyhow::bail!("No output file specified");
        }
    };

    let mut output_file = File::create(output_path)?;
    for line in BufReader::new(File::open(input_path)?).lines() {
        let data: DataIn = serde_json::from_str(&line?)?;

        let mut text = String::new();
        writeln!(text, "### Title:")?;
        writeln!(text)?;
        writeln!(text, "{}\n", normalize(&data.title))?;

        writeln!(text, "### Text:")?;
        writeln!(text)?;
        writeln!(text, "{}\n", normalize(&data.text))?;

        writeln!(text, "### Verdict:")?;
        writeln!(text)?;
        write!(text, "{}", data.verdict)?;

        let output = DataOut { text };
        let output = serde_json::to_string(&output)?;
        writeln!(output_file, "{}", output)?;
    }

    Ok(())
}

#[derive(Deserialize)]
struct DataIn {
    title: String,
    text: String,
    comments: Vec<String>,
    verdict: String,
}

#[derive(Serialize)]
struct DataOut {
    text: String,
}

fn normalize(s: &str) -> String {
    s.nfkd().collect::<String>()
}
