//! Produces the final training dataset.

use std::{io::BufRead, io::Write as IoWrite};

use rand::prelude::SliceRandom;
use serde::Serialize;
use unicode_normalization::UnicodeNormalization;
use xml::writer::XmlEvent;

use super::{Submission, Verdict, STAGE2_OUTPUT_PATH, STAGE3_OUTPUT_PATH};

#[derive(Serialize)]
struct DataOut {
    instruction: String,
    output: String,
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

        let mut text = vec![];
        let mut writer = xml::writer::EmitterConfig::new()
            .perform_indent(true)
            .write_document_declaration(false)
            .create_writer(&mut text);

        writer.write(XmlEvent::start_element("text"))?;
        writer.write(XmlEvent::characters(&normalize(&data.text)))?;
        writer.write(XmlEvent::end_element())?;

        let mut comments = data.comments.values().collect::<Vec<_>>();
        comments.sort_by_key(|c| -c.score);
        comments.truncate(5);
        comments.shuffle(&mut rand::thread_rng());

        writer.write(XmlEvent::start_element("comments"))?;
        for comment in comments {
            writer.write(XmlEvent::start_element("comment"))?;
            writer.write(XmlEvent::characters(&normalize(&comment.body)))?;
            writer.write(XmlEvent::end_element())?;
        }
        writer.write(XmlEvent::end_element())?;

        writer.write(XmlEvent::start_element("verdict"))?;
        writer.write(XmlEvent::characters(&match data.verdict {
            Verdict::NotTheAsshole => "NTA",
            Verdict::NoAssholesHere => "NAH",
            Verdict::EveryoneSucks => "ESH",
            Verdict::Asshole => "YTA",
        }))?;
        writer.write(XmlEvent::end_element())?;

        let output = serde_json::to_string(&DataOut {
            instruction: normalize(&data.title),
            output: String::from_utf8_lossy(&text).into_owned(),
        })?;
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
