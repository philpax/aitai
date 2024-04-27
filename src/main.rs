use std::collections::HashMap;

use anyhow::Context;
use serde::{Deserialize, Serialize};

mod stage1;
mod stage2;
mod stage3;

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
