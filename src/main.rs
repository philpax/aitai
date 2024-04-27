use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader},
};

use serde::Deserialize;
use zstd::stream::read::Decoder as ZstdDecoder;

const SUBMISSIONS_PATH: &str = "data/AmItheAsshole_submissions.zst";
const COMMENTS_PATH: &str = "data/AmItheAsshole_comments.zst";

#[derive(Deserialize, Debug)]
struct Submission {
    #[serde(default)]
    pub num_comments: usize,
    #[serde(default)]
    pub link_flair_text: Option<String>,
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub removed_by: Option<String>,
    #[serde(default)]
    pub selftext: String,
    #[serde(default)]
    pub sticked: bool,
    #[serde(default)]
    pub score: i32,
}

fn main() -> anyhow::Result<()> {
    let submissions = read_submissions()?;
    let comment_counts = get_comment_counts_for_submissions()?;
    dbg!(comment_counts);

    Ok(())
}

fn read_submissions() -> anyhow::Result<HashMap<String, Submission>> {
    zstd_bufreader(SUBMISSIONS_PATH)?
        .lines()
        .filter_map(Result::ok)
        .map(|line| {
            let submission = serde_json::from_str::<Submission>(&line)?;
            Ok((submission.id.clone(), submission))
        })
        .collect()
}

fn get_comment_counts_for_submissions() -> anyhow::Result<HashMap<String, usize>> {
    #[derive(Deserialize, Debug)]
    struct SimplifiedComment<'a> {
        link_id: &'a str,
    }

    Ok(zstd_bufreader(COMMENTS_PATH)?
        .lines()
        .filter_map(Result::ok)
        .map(|comment| {
            let comment = serde_json::from_str::<SimplifiedComment>(&comment).unwrap();
            comment.link_id.trim_start_matches("t3_").to_string()
        })
        .fold(HashMap::new(), |mut counts, submission_id| {
            *counts.entry(submission_id).or_insert(0) += 1;
            counts
        }))
}

fn zstd_bufreader(path: &str) -> anyhow::Result<impl BufRead> {
    Ok(BufReader::new(ZstdDecoder::new(BufReader::new(
        File::open(path)?,
    ))?))
}
