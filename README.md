# Am I The AIhole

A project to finetune a large language model using AITA posts to achieve two main objectives: generating complete posts solely from titles and auto-classifying posts based on titles and body text. By training the LLM on a dataset of AITA posts, the model will learn to generate detailed and coherent posts from titles alone and classify posts accurately.

This should not be used seriously. The model will likely be biased and will not be able to generate coherent posts/classify posts accurately. This is a toy project to experiment with large language models and their capabilities.

A subset of the comments are included within the dataset to help the model "come to a decision"; the idea is that this will resemble chain-of-thought prompting and help the model generate a more grounded verdict.

# Running

The scripts and dataset outputter need to be run in order. The scripts use single-threaded jq and could be a lot faster (i.e. RIIR). Here are the rough commands used (running in this directory):

```sh
./scripts/0-download.sh
./scripts/2-comments-extract.sh data/AmItheAsshole_comments.zst data/extracted_comments.ndjson
./scripts/2-submissions-extract.sh data/AmItheAsshole_submissions.zst data/extracted_submissions.ndjson
./scripts/3-comments-group-by-submissions.sh data/extracted_comments.zst data/grouped_extracted_comments.ndjson
./scripts/4-combine-submissions-and-comments.sh data/extracted_submissions.ndjson data/grouped_extracted_comments.ndjson data/submissions_and_comments.ndjson
cd dataset-outputter
cargo run --release -- ../data/submissions_and_comments.ndjson ../data/output.ndjson
```

That will produce a dataset. Next, use Docker and Axolotl to tokenise the dataset and train on it (again, in this directory); this assumes the use of a single 3090 being used for a full-finetune, adjust as necessary:
```sh
docker run --privileged --gpus '"all"' --rm -it --name axolotl --mount type=bind,src="${PWD}",target=/workspace/axolotl/aitai -v ${HOME}/.cache/huggingface:/root/.cache/huggingface winglian/axolotl:main-latest

# stablelm-2-1-6b-fft.yml / phi-2-lora.yml / llama-3-8b-lora.yml
CUDA_VISIBLE_DEVICES="" python -m axolotl.cli.preprocess aitai/axolotl-configs/stablelm-2-1-6b-fft.yml
python -m axolotl.cli.train aitai/axolotl-configs/stablelm-2-1-6b-fft.yml
```

### Scripts

The `scripts` process data from the r/AITA dumps (submissions, comments) from <https://the-eye.eu/redarcs/>.

Stage 0 scripts download the raw data.

Stage 1 scripts take a sample from the raw dumps for experimentation.

Stage 2 scripts can take a Stage 1 sample or the raw dumps. Their job is to extract all relevant submissions and comments and discard all irrelevant data.

Stage 3 scripts take the result of Stage 2 scripts and extract any metadata required for later.

Stage 4 scripts take the output of Stage 2 and Stage 3 scripts and combine them.

### Dataset Outputter

The dataset outputter takes the output of Stage 4 and produces the final dataset ndjson to train on using [axolotl](https://github.com/OpenAccess-AI-Collective/axolotl).

