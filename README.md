# Am I The AIhole

A project to finetune a large language model using AITA posts to achieve two main objectives: generating complete posts solely from titles and auto-classifying posts based on titles and body text. By training the LLM on a dataset of AITA posts, the model will learn to generate detailed and coherent posts from titles alone and classify posts accurately.

This should not be used seriously. More details may come in a blog post; I'm still working on training.

A subset of the comments are included within the dataset to help the model "come to a decision"; the idea is that this will resemble chain-of-thought prompting and help the model generate a more grounded verdict.

## Scripts

The `scripts` process data from the r/AITA dumps (submissions, comments) from <https://the-eye.eu/redarcs/>.

Stage 0 scripts take a sample from the raw dumps for experimentation.

Stage 1 scripts can take a Stage 0 sample or the raw dumps. Their job is to extract all relevant submissions and comments and discard all irrelevant data.

Stage 2 scripts take the result of Stage 1 scripts and extract any metadata required for later.

Stage 3 scripts take the output of Stage 1 and Stage 2 scripts and combine them.

## Dataset Outputter

The dataset outputter takes the output of Stage 3 and produces the final dataset ndjson to train on using [axolotl](https://github.com/OpenAccess-AI-Collective/axolotl).