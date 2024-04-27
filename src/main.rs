use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};

const SUBMISSIONS_PATH: &str = "data/AmItheAsshole_submissions.zst";
const COMMENTS_PATH: &str = "data/AmItheAsshole_comments.zst";
const STAGE1_OUTPUT_PATH: &str = "data/stage1_output.ndjson";
const STAGE2_OUTPUT_PATH: &str = "data/stage2_output.ndjson";
const STAGE3_OUTPUT_PATH: &str = "data/stage3_output.ndjson";
const MAX_SUBMISSIONS_PER_VERDICT: usize = 500;

#[derive(Deserialize, Serialize, Debug)]
struct Submission {
    pub title: String,
    pub text: String,
    pub score: i32,
    pub verdict: Verdict,
    pub comments: HashMap<String, Comment>,
}
#[derive(Deserialize, Serialize, Debug)]
struct Comment {
    body: String,
    score: i32,
}
#[derive(Deserialize, Serialize, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
enum Verdict {
    NotTheAsshole,
    NoAssholesHere,
    EveryoneSucks,
    Asshole,
}

fn main() -> anyhow::Result<()> {
    let first_arg = std::env::args()
        .nth(1)
        .context("missing argument; must be stage1 or stage2")?;
    match first_arg.as_str() {
        "stage1" => stage1::run()?,
        "stage2" => stage2::run()?,
        "stage3" => stage3::run()?,
        _ => anyhow::bail!("invalid argument; must be stage1 or stage2"),
    }

    Ok(())
}

mod stage1 {
    //! Ingests all submissions and comments, ignoring the submissions that are
    //! immediately irrelevant to our task, and then outputs the submissions with
    //! their comments to a new file.

    use std::{
        borrow::Cow,
        collections::HashMap,
        fs::File,
        io::{BufRead, BufReader, Write},
    };

    use serde::Deserialize;
    use zstd::stream::read::Decoder as ZstdDecoder;

    use super::{
        Comment, Submission, Verdict, COMMENTS_PATH, STAGE1_OUTPUT_PATH, SUBMISSIONS_PATH,
    };

    pub fn run() -> anyhow::Result<()> {
        println!("Submissions: reading");
        let start = std::time::Instant::now();
        let mut submissions = read_submissions()?;
        println!(
            "Submissions: read {} items ({}s)",
            submissions.len(),
            start.elapsed().as_secs_f32(),
        );

        println!("Comments: populating");
        let start = std::time::Instant::now();
        populate_comments(&mut submissions)?;
        println!("Comments: populated ({}s)", start.elapsed().as_secs_f32());

        println!("Submissions: writing");
        let start = std::time::Instant::now();
        let file = std::fs::File::create(STAGE1_OUTPUT_PATH)?;
        let mut writer = std::io::BufWriter::new(file);
        for submission in submissions.values() {
            writeln!(writer, "{}", serde_json::to_string(submission)?)?;
        }
        println!("Submissions: written ({}s)", start.elapsed().as_secs_f32());

        Ok(())
    }

    fn read_submissions() -> anyhow::Result<HashMap<String, Submission>> {
        #[derive(Deserialize, Debug)]
        struct SubmissionRaw<'a> {
            pub title: Cow<'a, str>,
            #[serde(default)]
            pub num_comments: usize,
            #[serde(default)]
            pub link_flair_text: Option<Cow<'a, str>>,
            #[serde(default)]
            pub id: Cow<'a, str>,
            #[serde(default)]
            pub removed_by: Option<Cow<'a, str>>,
            #[serde(default)]
            pub selftext: Cow<'a, str>,
            #[serde(default)]
            pub sticked: bool,
            #[serde(default)]
            pub score: i32,
        }

        Ok(zstd_bufreader(SUBMISSIONS_PATH)?
            .lines()
            .filter_map(Result::ok)
            .flat_map(|l| {
                let s = serde_json::from_str::<SubmissionRaw>(&l).unwrap();
                let verdict = parse_verdict(s.link_flair_text?.as_ref())?;

                if s.num_comments == 0
                    || s.removed_by.is_some()
                    || ["[removed]", "[deleted]"].contains(&s.selftext.as_ref())
                    || s.sticked
                {
                    return None;
                }

                Some((
                    s.id.into_owned(),
                    Submission {
                        title: s.title.into_owned(),
                        text: s.selftext.into_owned(),
                        score: s.score,
                        verdict,
                        comments: Default::default(),
                    },
                ))
            })
            .collect())
    }

    fn parse_verdict(value: &str) -> Option<Verdict> {
        match value {
            "Not the A-hole" => Some(Verdict::NotTheAsshole),
            "No A-holes here" => Some(Verdict::NoAssholesHere),
            "Everyone Sucks" => Some(Verdict::EveryoneSucks),
            "Asshole" => Some(Verdict::Asshole),
            _ => None,
        }
    }

    fn populate_comments(submissions: &mut HashMap<String, Submission>) -> anyhow::Result<()> {
        #[derive(Deserialize, Debug)]
        struct CommentRaw<'a> {
            link_id: Cow<'a, str>,
            body: Cow<'a, str>,
            score: i32,
        }

        for line in zstd_bufreader(COMMENTS_PATH)?
            .lines()
            .filter_map(Result::ok)
        {
            let comment = serde_json::from_str::<CommentRaw>(&line).unwrap();
            let submission_id = comment.link_id.trim_start_matches("t3_");

            if let Some(submission) = submissions.get_mut(submission_id) {
                submission.comments.insert(
                    submission_id.to_string(),
                    Comment {
                        body: comment.body.into_owned(),
                        score: comment.score,
                    },
                );
            }
        }

        Ok(())
    }

    fn zstd_bufreader(path: &str) -> anyhow::Result<impl BufRead> {
        Ok(BufReader::with_capacity(
            10 * 1_024 * 1_024,
            ZstdDecoder::new(File::open(path)?)?,
        ))
    }
}

mod stage2 {
    //! Buckets all of the submissions by their verdicts, sorts them by some metric, and then selects the top N
    //! submissions from each bucket to output.

    use std::{
        collections::HashMap,
        io::{BufRead, Write as IoWrite},
    };

    use rand::prelude::SliceRandom;

    use super::{
        Submission, Verdict, MAX_SUBMISSIONS_PER_VERDICT, STAGE1_OUTPUT_PATH, STAGE2_OUTPUT_PATH,
    };

    pub fn run() -> anyhow::Result<()> {
        let mut buckets: HashMap<Verdict, Vec<Submission>> = Default::default();

        // Ingest all data into buckets
        println!("Submissions: reading");
        let now = std::time::Instant::now();
        for line in std::io::BufReader::new(std::fs::File::open(STAGE1_OUTPUT_PATH)?).lines() {
            let submission: Submission = serde_json::from_str(&line?)?;
            buckets
                .entry(submission.verdict)
                .or_default()
                .push(submission);
        }
        println!("Submissions: read ({}s)", now.elapsed().as_secs_f32());

        // Sort and prune the buckets
        println!("Submissions: sorting");
        let now = std::time::Instant::now();
        for bucket in buckets.values_mut() {
            bucket.sort_by_key(|submission| submission.score + submission.comments.len() as i32);
            bucket.truncate(MAX_SUBMISSIONS_PER_VERDICT);
        }
        println!("Submissions: sorted ({}s)", now.elapsed().as_secs_f32());

        // Collect all of the submissions once more
        println!("Submissions: collecting");
        let now = std::time::Instant::now();
        let mut output_file = std::fs::File::create(STAGE2_OUTPUT_PATH)?;
        let mut data = buckets
            .values_mut()
            .flat_map(|v| v.drain(..))
            .collect::<Vec<_>>();
        data.shuffle(&mut rand::thread_rng());
        println!("Submissions: collected ({}s)", now.elapsed().as_secs_f32());

        // Output them
        println!("Submissions: writing");
        let now = std::time::Instant::now();
        for submission in &data {
            let output = serde_json::to_string(&submission)?;
            writeln!(output_file, "{}", output)?;
        }
        println!(
            "Submissions: written {} items ({}s)",
            data.len(),
            now.elapsed().as_secs_f32()
        );

        Ok(())
    }
}

mod stage3 {
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
}
