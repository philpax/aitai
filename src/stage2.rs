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
