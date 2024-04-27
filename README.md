# Am I The AIhole

A project to finetune a large language model using AITA posts to achieve two main objectives: generating complete posts solely from titles and auto-classifying posts based on titles and body text. By training the LLM on a dataset of AITA posts, the model will learn to generate detailed and coherent posts from titles alone and classify posts accurately.

This should not be used seriously. The model will likely be biased and will not be able to generate coherent posts/classify posts accurately. This is a toy project to experiment with large language models and their capabilities.

A subset of the comments are included within the dataset to help the model "come to a decision"; the idea is that this will resemble chain-of-thought prompting and help the model generate a more grounded verdict.

# Running

The raw data needs to be downloaded using `./download.sh`, and then the Rust code needs to be run in order to produce the training dataset.

```sh
./download.sh
cargo run --release -- stage1
cargo run --release -- stage2
cargo run --release -- stage3
```

That will produce a dataset. Next, use Docker and Axolotl to tokenise the dataset and train on it (again, in this directory); this assumes the use of a single 3090 being used for a full-finetune, adjust as necessary:
```sh
docker run --privileged --gpus '"all"' --rm -it --name axolotl --mount type=bind,src="${PWD}",target=/workspace/axolotl/aitai -v ${HOME}/.cache/huggingface:/root/.cache/huggingface winglian/axolotl:main-latest

# stablelm-2-1-6b-fft.yml / phi-2-lora.yml / llama-3-8b-lora.yml
CUDA_VISIBLE_DEVICES="" python -m axolotl.cli.preprocess aitai/axolotl-configs/stablelm-2-1-6b-fft.yml
python -m axolotl.cli.train aitai/axolotl-configs/stablelm-2-1-6b-fft.yml
```
