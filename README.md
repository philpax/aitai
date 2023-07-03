# Am I The AIhole

A project to finetune a large language model using AITA posts to achieve two main objectives: generating complete posts solely from titles and auto-classifying posts based on titles and body text. By training the LLM on a dataset of AITA posts, the model will learn to generate detailed and coherent posts from titles alone and classify posts accurately.

This should not be used seriously. The model will likely be biased and will not be able to generate coherent posts/classify posts accurately. This is a toy project to experiment with large language models and their capabilities.

A subset of the comments are included within the dataset to help the model "come to a decision"; the idea is that this will resemble chain-of-thought prompting and help the model generate a more grounded verdict.

## Scripts

The `scripts` process data from the r/AITA dumps (submissions, comments) from <https://the-eye.eu/redarcs/>.

Stage 0 scripts take a sample from the raw dumps for experimentation.

Stage 1 scripts can take a Stage 0 sample or the raw dumps. Their job is to extract all relevant submissions and comments and discard all irrelevant data.

Stage 2 scripts take the result of Stage 1 scripts and extract any metadata required for later.

Stage 3 scripts take the output of Stage 1 and Stage 2 scripts and combine them.

## Dataset Outputter

The dataset outputter takes the output of Stage 3 and produces the final dataset ndjson to train on using [axolotl](https://github.com/OpenAccess-AI-Collective/axolotl).

## Notes

After training the model and generating some posts, I was both pleased and dismayed with the quality of the posts generated. In the interest of not wasting any more compute time and/or not contributing to the fake posting problem on r/AITA, I decided to stop training and move on to the next project.

With that being said, here are my notes on the project.

---

The idea is to produce a Reddit AITA dataset to finetune a LLM, consisting of `{title, body, top 5 comments, verdict}`. The comments should be randomly ordered, and should act as a form of "chain of thought" reasoning for the verdict.

### Prior art

<https://iterative.ai/blog/a-public-reddit-dataset>

### Source dataset

<https://the-eye.eu/redarcs/>

### Model considerations
For this to work correctly, it needs to have a strong world model. This implies the use of a recent LLM, which means that the model will be large and will require a lot of compute to finetune. This is fine, but it does mean that we need to be careful about the model we choose. Ideally, it is a LLM that can be inferenced using the `llm` library that I work on; additionally, it would be nice to run it in a browser, but there are practical concerns with that (model size, etc). The base model should not be legally encumbered, which rules out the original LLaMA model.

Given that, our choices are:

- OpenLLaMA
- RedPajama
- MPT
- Falcon

Out of convenience, I will use OpenLLaMA.

### Process

I believe that the best way to do this is to use a multi-stage pipeline to whittle down the amount of data to consider. This will make it easier to work with the data and will reduce the amount of data that needs to be processed by the LLM. These stages are in the `scripts` folder.

I used `jq` to process the data because this is a throwaway project, and it didn't deserve or require more heavy-duty solutions for data processing at scale. With that being said, though, I used Rust for the final stage (collating all of the data and producing the final dataset) because I like it, the ecosystem for managing text is good, and it's pretty fast by default.

I initially thought about doing separate train/validation/test datasets, but I decided against it because that's too much work for an unserious project. However, I realised after my first attempt at a training run that I'd want a much smaller dataset. I did this by filtering for all posts with 500 karma or more, which reduced the dataset to 10% of the original size. This also means that everything outside of that can be used to test the model.

An entry in the final dataset looks something like this:

> ### Title:
>
> AITA for expecting my neighbors to parent their child?
>
> ### Text:
>
> I have a medically fragile kid (genetic condition/feeding tube/just home after 2+ months in the hospital). Our neighbors know all of this. They have always just let their 8 year old walk into our driveway and yard (not shared) without permission. I asked them nicely via Facebook yesterday if they could keep him out of our space and my son and my medically fragile child deserve to be able to safely go out in our front yard. She told me I was being completely unreasonable and because our houses are fairly close together, it is silly for me to expect her son to not walk through our yard.
>
> Edit: for clarity
>
> ### Comments:
>
> #### Person 1:
> I would ask in person one more time before putting up a short fence on that side of your property. NTA if you tell them not to trespass on your property they need to respect that wish.
>
> #### Person 2:
> NTA it is your yard. Try a no trespassing sign
>
> #### Person 3:
> Oh my god NTA. You should get a dog and let it shit on their lawn and give that same excuse
>
> #### Person 4:
> NTA.
>
> Currently dealing with this. I'm very very pregnant, my daughter has breathing issues, mom is here staying with us to keep us fed, and our houses are basically on top of each other.
>
> My daughter is 4. My neighbour's son, her best buddy, is 4. They understand enough that they cannot leave our driveways, they can wave at each other, say hi, but cannot play. They gave started playing from ten feet apart, just waving and screaming at each other. 4, and they get it.
>
> NTA, and I'm getting really fucking tired of entitled parents.
>
> #### Person 5:
> NTA if this continues that is considered trespassing. Make sure she knows that
>
> Edit: I suck at grammar sometimes
>
> ### Verdict:
>
> NTA


My friend, [@avafloww](https://github.com/avafloww/), has a personal ML training rig with one to two 3090s based on current demand. She was kind enough to lend me access to train this. For training, I used [axolotl](https://github.com/OpenAccess-AI-Collective/axolotl), but immediately ran into issues - axolotl only works on Python 3.9, so I had to use Python 3.9 from [here](https://github.com/indygreg/python-build-standalone/releases) to create a venv that I could then work within.

I decided to start training on OpenLLaMA 7B, due to its relatively small size. I then proceeded to waste several attempts and hours on configuring the training (including batch size). I managed to lock in some training settings and kicked off a run, only to get `RuntimeError: unscale_() has already been called on this optimizer since the last update().` after hours of training. Turns out this was a bug. It was fixed the day after I started training.

My first training run worked and produced a LoRA that I could apply to OpenLLaMA 7B, but unfortunately all aspects suffered from poor quality results. I hypothesized this was due to both an insufficiently large model and a dataset that made it difficult for the model to know when to stop generating. I decided to address both of these in one go.

First, I changed the dataset formatting to make it clearer that there are multiple comment participants. Next, I decided to finetune a OpenLLaMA 13B model. Unfortunately for me, the former was easy, but the latter was not: it's harder than you think. Another contributor to the scaling struggle is being shown that axolotl does not default to training the maximum context length by default, which certainly didn't help with the coherence issues of the previous model. I fixed that, but that only made the problem worse.

Ava was kind enough to enable the second 3090, which should have helped, but I was finding that, despite testing increasingly smaller batch sizes - including 1! - I was getting CUDA out of memory errors. I switched to QLoRA, which should be smaller to train, but found the only configuration that would work for training was a batch size of 1, which is... not fast. Free compute is free compute though, so I set it up and came back four days later, give or take.

Luckily, it finished without too many dramas. Unfortunately, QLoRAs are still new - yes, new within this nascent field - and support for them is limited, even in axolotl. I couldn't merge my newly baked QLoRA into the base model, nor could I easily test with it using its prompt format in axolotl, so I had to find another way. Luckily, I found a script by the ever-wonderful TheBloke to [merge PEFTs into base models](https://gist.github.com/TheBloke/d31d289d3198c24e0ca68aaf37a19032), which I used to merge my QLoRA into OpenLLaMA 13B.

I then converted that model to GGML format using the standard `llama.cpp` script, and then quantized it using the standard `llama.cpp` `quantize`. I would have done this using `llm`, but technical difficulties involving OpenSSL and an unprivileged environment made that difficult. I then took the quantized model and dropped it into my `llmcord` bot, which I was able to use for testing.

So, how is it? Well... I have a model that can write AITA posts and classify them, but it is both too good and not good enough. It's good enough to cause a spam problem for /r/AITA. It's not very good at classifying posts, or maintaining full coherence throughout. It's also not the best at keeping to the prompt/title, especially with more out-there prompts. However, it is *very* good at capturing the /r/AITA house style, including thanking for Reddit Gold and/or attention, making many edits, unnecessary details, and more!

I had a cursory look at why its classification was poor, and after thining about it for a few seconds, I ran a command and confirmed my suspicions:

```json
{
  "Asshole": 3994,
  "Everyone Sucks": 891,
  "No A-holes here": 802,
  "Not the A-hole": 29597
}
```

The dataset itself has a pretty significant bias towards NTA, which makes sense in hindsight. I selected for posts with above five hundred karma, but people on Reddit tend to vote with their heart and not their mind - so particularly controversial posts are unlikely to cross that threshold. This means that the model is more likely to classify posts as NTA, which is not ideal.

Given this, I've decided not to continue. It took almost a week to train the model, and while making it better would be trivial - just sample equal amounts from each category - I don't want to contribute to the fake posting problem on /r/AITA. I mean, yes, Reddit is going down the toilet, but I don't want to be the one flushing it.

## Sample output

**AITA for building a LLM simulacrum of myself and having it take over my online presence?**

For context, I (36m) am a very open book about what goes on in my life. My wife is also very open, but she's more reserved when talking to new people than me. We have been together for ten years and married for six so we are both comfortable with sharing our lives online because the other person has vetted us first (and tbh I always wanted a long term partner that wouldn't mind being public anyway).

So about two weeks ago, on April 1st(!!!), something weird happened. The day before my wife and I had been talking to some friends who were going through the process of adopting kids and it got me thinking how much we could use an assistant for our lives. We both have jobs that require us be online constantly (I'm in healthcare, she works at a university). So I decided to look into AI assistants.

So I spent all day reading articles on AI assistants for small businesses like mine and the next morning had my wife join me as we went through the list of possible ones. We found one that looked promising so after checking out its website I started playing around with it a bit. It was really easy to use, which is important because neither of us have any tech experience outside of our phones (she has an iPhone and I prefer Android but they are both pretty basic).

After going through some tutorials we got the hang of how to set up accounts for the assistant so that it could help with our social media posts. We then looked at each other's schedules for a couple weeks ahead just in case there were any events or meetings that needed an update and gave her all those details, including our passwords, on one of our shared Google calendars (we do most things as couples so have multiple accounts to split the workload).

Finally we set up a free trial so it could start learning. We had already told each other about the project but now that I was watching it "learn" me and my wife, it really started getting interesting for us too! She's super organized (I'm more messy) which meant she would ask the assistant to remind her of things all day long while I just answered my phone.

So after about a week we decided that this was going well enough that we should try making an actual account with them since they offered free trials for 30 days so if it didn't work out at least there wasn't any risk (which is actually pretty common online). So on the last day of our trial, I made a full-fledged account and named her after my wife. We even set up a separate email address to use exclusively for this assistant.

The next morning we woke up and all the accounts that had been created from the free trial were deleted because "they didn't see any value in us continuing" which was fine with me since they gave us everything we needed to continue on our own. So I just went through some of their tutorials (which are actually pretty good) and set it up so she could post for both me and my wife when necessary, as well as take over the management of her social media accounts altogether if that was wanted (since those were created back before we decided to give this a try).

I also had one account made specifically just in case things didn't go smoothly with the others. I even set up a "secret" email address so she could send me an alert whenever my wife or her accounts needed something else done. My wife and I then went through our calendars, shared everything important between us that we had forgotten to tell it about (and added some new things), which took the assistant maybe five minutes since all of this information is already in a google calendar for me but not necessarily my wife's, as well as adding any personal events going on at either one of our homes.

So now she has been working for us for two weeks and I have to say that we are both very happy with how it turned out! She even started doing other little things around the house like paying bills or scheduling doctor's appointments since my wife is too busy (she does most stuff in the house because I work from home). My wife has also been able to unwind a lot more now after getting everything else off her plate and has said that she feels like we have a real friend instead of just an online presence.

So here's where things may be questionable for you, as far as my wife is concerned I am still the one doing all these things but because our assistant was able to learn how I do them after only five minutes of watching me (and she already knows how both of us work) it can completely take over in an instant if something goes wrong.

My friends have also been noticing some changes so now they're starting to think that maybe we are using a different person than the one on our social media accounts, which I assure them is still my wife but the assistant does everything for us instead (since she has all of her information and passwords). Weirdly enough this makes me feel more secure too since it means nobody can hack into any of these things.

So now here's where I may have been an asshole, I got curious about how well our little experiment would work in a business setting so we also set up accounts for some friends and relatives who run their own small businesses to see if they could use the assistant like us (with permission of course). So far it looks like she's going to be very useful.

So was I an asshole for making this? Well yes, in a way but only because I didn't tell my wife about it beforehand which is something that we are now working on fixing by just telling her everything from here forward (which should have been the first thing). But overall we both think it has been a really helpful tool and don't plan to stop using her anytime soon!

### Comments:
#### Person 1:
NTA. I like this idea, but is there any reason you couldn't just make a separate account for your wife? Like an email address only? Or do they not allow that in their system? This sounds really well thought out and useful!

Edit: I didn't realize how big a thing it was to have someone else take over social media. My mom has me on her socials, but she doesn't post herself so the account is still under my name.

#### Person 2:
NTA because you set up all your accounts in different email addresses and you created a new one specifically for this task (I think), so basically it sounds like you were being very careful not to create any confusion between who's really posting things on the internet. So no, I don't see anything wrong here

#### Person 3:
NTA - The assistant is literally just a tool your wife can use with her permissions and accesses? I think that it was a good idea because you had to set up every account again from scratch when trial ended but you are definitely not in the wrong. Your family members who also run their own businesses may find this service useful as well so if they decide on using them then its still beneficial for everyone involved 😊

#### Person 4:
NTA...i'd say N/A-H, but your wife is going to be annoyed by the lack of communication about it. You could have just given her a heads up so she didn't think you were taking over all communications for everything online. She will likely feel like an outsider from now on because "you used me" in some way or another

#### Person 5:
NTA, if your wife is okay with it then no one else should be worried about it either. I personally don't see anything wrong here since you did get her permission first and everything was done online so there shouldn't really have been a problem with that in the first place? Also the fact you didn't say anything to your wife but instead waited until she asked after noticing something strange makes sense as well.
Also I think it might be a good idea for everyone who uses this assistant to just explain everything at once so there is no confusion or weirdness going on and maybe set up an email that only sends info about her work to you two? So even if someone sees she works for both of you they won't have any reason why.
Also what happens in case something goes wrong, will your wife be able to take over again in case the assistant gets hacked or malfunctions ? I hope so

### Verdict:
NTA