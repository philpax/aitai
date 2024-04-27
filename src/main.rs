use std::{
    borrow::Cow,
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
};

use serde::{Deserialize, Serialize};
use zstd::stream::read::Decoder as ZstdDecoder;

const SUBMISSIONS_PATH: &str = "data/AmItheAsshole_submissions.zst";
const COMMENTS_PATH: &str = "data/AmItheAsshole_comments.zst";
const SUBMISSIONS_AND_COMMENTS_OUTPUT_PATH: &str = "data/submissions_and_comments.ndjson";

fn main() -> anyhow::Result<()> {
    println!("Submissions: reading");
    let start = std::time::Instant::now();
    let mut submissions = read_submissions()?;
    println!(
        "Submissions: read at {}s, {} items",
        start.elapsed().as_secs_f32(),
        submissions.len()
    );

    println!("Comments: populating");
    let start = std::time::Instant::now();
    populate_comments(&mut submissions)?;
    println!("Comments: populated at {}s", start.elapsed().as_secs_f32());

    println!("Submissions: writing");
    let start = std::time::Instant::now();
    let file = std::fs::File::create(SUBMISSIONS_AND_COMMENTS_OUTPUT_PATH)?;
    let mut writer = std::io::BufWriter::new(file);
    for submission in submissions.values() {
        writeln!(writer, "{}", serde_json::to_string(submission)?)?;
    }
    println!("Submissions: written at {}s", start.elapsed().as_secs_f32());

    Ok(())
}

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

#[derive(Deserialize, Serialize, Debug)]
enum Verdict {
    NotTheAsshole,
    NoAssholesHere,
    EveryoneSucks,
    Asshole,
}
impl TryFrom<&str> for Verdict {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "Not the A-hole" => Ok(Verdict::NotTheAsshole),
            "No A-holes here" => Ok(Verdict::NoAssholesHere),
            "Everyone Sucks" => Ok(Verdict::EveryoneSucks),
            "Asshole" => Ok(Verdict::Asshole),
            _ => Err(()),
        }
    }
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
            let verdict = Verdict::try_from(s.link_flair_text.as_ref()?.as_ref()).ok()?;

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
