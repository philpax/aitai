//! Produces the final training dataset.

use std::{fmt::Write, io::BufRead, io::Write as IoWrite};

use rand::prelude::SliceRandom;
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;

use super::{Submission, Verdict, STAGE2_OUTPUT_PATH, STAGE3_OUTPUT_PATH};

#[derive(Serialize)]
struct DataOut {
    text: String,
}

pub fn run() -> anyhow::Result<()> {
    let mut output_file = std::fs::File::create(STAGE3_OUTPUT_PATH)?;
    println!("Submissions: processing");
    let now = std::time::Instant::now();
    for (index, line) in std::io::BufReader::new(std::fs::File::open(STAGE2_OUTPUT_PATH)?)
        .lines()
        .enumerate()
    {
        if index % 1_000 == 0 {
            println!(
                "Submissions: processed {} at {}s",
                index,
                now.elapsed().as_secs_f32()
            );
        }

        let data: Submission = serde_json::from_str(&line?)?;

        let mut text = String::new();
        writeln!(text, "### Title:")?;
        writeln!(text)?;
        writeln!(text, "{}\n", normalize(&data.title))?;

        writeln!(text, "### Text:")?;
        writeln!(text)?;
        writeln!(text, "{}\n", normalize(&data.text))?;

        writeln!(text, "### Comments:")?;
        writeln!(text)?;

        let mut comments = data.comments.values().collect::<Vec<_>>();
        comments.sort_by_key(|c| -c.score);
        comments.truncate(5);
        comments.shuffle(&mut rand::thread_rng());

        for (idx, comment) in comments.iter().enumerate() {
            writeln!(text, "#### Person {}:", idx + 1)?;
            writeln!(text, "{}", normalize(&comment.body))?;
            writeln!(text)?;
        }

        writeln!(text, "### Verdict:")?;
        writeln!(text)?;
        write!(
            text,
            "{}",
            match data.verdict {
                Verdict::NotTheAsshole => "NTA",
                Verdict::NoAssholesHere => "NAH",
                Verdict::EveryoneSucks => "ESH",
                Verdict::Asshole => "YTA",
            }
        )?;

        let output = serde_json::to_string(&DataOut { text })?;
        writeln!(output_file, "{}", output)?;
    }
    println!(
        "Submissions: processed all at {}s",
        now.elapsed().as_secs_f32()
    );

    Ok(())
}

fn normalize(s: &str) -> String {
    let mut output = String::new();
    for c in html_escape::decode_html_entities(&s)
        .replace("&#x200B;", "")
        .nfkd()
    {
        match c {
            '‘' => output.push_str("\'"),
            '’' => output.push_str("\'"),
            '“' => output.push_str("\""),
            '”' => output.push_str("\""),
            '–' => output.push_str("-"),
            '—' => output.push_str("-"),
            '…' => output.push_str("..."),
            c => output.push(c),
        }
    }
    output
}
