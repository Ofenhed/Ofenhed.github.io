use crate::{
    blog::metadata::{BlogEntry, Locale, Tag},
    helpers::{Abbr, Footnote, Url},
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_meta::Style;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct WhatAreLLMs;

impl BlogEntry for WhatAreLLMs {
    fn uid() -> u32 {
        4
    }

    fn publish() -> bool {
        true
    }

    fn publish_date() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-06-06T12:00:00+01:00")
            .unwrap()
            .into()
    }

    fn last_updated() -> Option<DateTime<Utc>> {
        None
    }

    fn locale() -> Option<Locale> {
        Locale::EnglishSimplified.into()
    }

    fn title() -> &'static str {
        "An intuitive understanding of LLM"
    }

    fn tags() -> &'static [Tag] {
        &[Tag::Ai, Tag::Tech]
    }
}

#[lazy_route]
impl LazyRoute for WhatAreLLMs {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let ai = || view! { <Abbr title="Artificial Intelligence">"AI"</Abbr> }.into_inner();
        let llm = || view! { <Abbr title="Large Language Model">"LLM"</Abbr> }.into_inner();
        let llms = || {
            view! {
                <Abbr title="Large Language Model" suffix="s">
                    "LLM"
                </Abbr>
            }
            .into_inner()
        };
        let agi =
            || view! { <Abbr title="Artificial General Intelligence">AGI</Abbr> }.into_inner();
        view! {
            <Style>
                "@media screen {ul.lies em {color: #f00; text-shadow: 0 0 0.8em light-dark(#000, #555);}}"
            </Style>
            <section>
                "When we talk about "{ai}" today, we pretty much talk about "{llms}
                ". We're being told that they are able to (or very soon able to) fully replace some human workers. I'm writing this to help put some context to that statement. I'm trying to be concise; my priority with this text is for it to be accessible."
            </section>
            <section>
                "To be able to understand what an "{llm}
                " is, we'll take a detour to compression algorithms. "<details>
                    <summary>
                        "There are two categories of compression: lossless compression and lossy compression."
                    </summary>
                    <b>"Lossless compression"</b>
                    " means that you will always get back the exact data you had before compression. You're using this right now, in your browser. It's most often done by applying known predefined algorithms designed to find and efficiently describe patterns. The compression ratio is decided by the effectiveness of the algorithm and how much computing power and memory you provide."
                    <br />
                    <b>"Lossy compression"</b>
                    " means that you lose data when you compress. This is often done by looking for things that can be removed without us noticing, such as high pitch sounds and dark colors, in combination with approximations of patterns. The compression ratio here is decided by the same as lossless compression, but also by how much quality degradation you can accept. This is what's used in most photographs and videos, and is what's responsible for JPEG images sometimes being blocky."
                </details>
                "Historically, text has almost exclusively been used with lossless compression. "
                {llms}" are a revolutionary change to that. "{llms}
                " are not compression in the traditional sense, where you'd expect the compressed file to grow when you give it more data, compressed by a predefined set of algorithms. With "
                {llms}
                " you create the output file from the start, then you take an arbitrary amount of data and optimize the parameters in that existing output file to be able to approximate the data you provide."
            </section>
            <section>
                "I get that some of you will have an instant reaction of this comparison being ridiculous. I would like to direct you to a study"
                <Footnote>
                    <b>Extracting books from production language models</b>
                    <br />
                    <i>"Ahmed Ahmed, A. Feder Cooper, Sanmi Koyejo, Percy Liang"</i>
                    <br />
                    <Url>"https://arxiv.org/pdf/2601.02671v1"</Url>
                </Footnote>" that could get "{llm}
                " models to recall (near-verbatim) 95.8% of the first Harry Potter book and 95.5% of 1984. This doesn't prove lack of intelligence, but it proves that the data is there."
            </section>
            <section>
                <h2>"Where does the intelligence come from?"</h2>
                "I have identified the following sources:"
                <ul>
                    <li>
                        <b>"Volume"</b>
                        ": Pretty much all text humanity has ever written has been used to optimize these models. Your query may not be as unique as you think."
                    </li>
                    <li>
                        <b>"Huge context windows"</b>
                        ": The generated text is put together based on a ridiculous number of parameters. In combination with the volume of data, it will mostly output coherent text, even when the text is completely nonsensical."
                    </li>
                    <li>
                        <b>
                            "Idealization/"
                            <a href="https://en.wikipedia.org/wiki/Apophenia">Apophenia</a>
                        </b>
                        ": "
                        {llms}
                        " are really good at making you feel listened to. If you talk to "
                        {llms}
                        " as you would a friend, then I would expect that you want there to be a meaning to those conversations."
                    </li>
                    <li>
                        <b>"Authority"</b>
                        ": These techniques are developed by the richest people in the world. They can't all be lying, right? "
                        <i>
                            "(This is obviously written from my Tesla Roadster while autonomously riding along the open roads on Mars)"
                        </i>
                    </li>
                    <li>
                        <b>"Trojan horses"</b>
                        ": There is a lot of propaganda that sounds like criticism against "
                        {ai}
                        ", which in actuality sneakily promote "
                        {ai}
                        <Footnote>
                            "AI companies keep telling us about how dangerous AI can be, they even ask for new laws. Anthropic claims that "
                            <a href="https://www.ynetnews.com/tech-and-digital/article/hkftl9ibmg">
                                "Claude Mythos AI is too dangerous for the public"
                            </a>
                            ". They warn that the models might at any time start improving themselves without human help, become super intelligent, and take over the world. "
                            <em>
                                "All of these claims strengthens the confidence in "{llm}
                                " as a road to intelligence. It also keeps lawmakers focused on sci-fi threats, instead of the damage the "
                                {ai}" hype is already doing."
                            </em>
                        </Footnote>
                        "."
                    </li>
                    <li>
                        <b>
                            <a href="https://en.wiktionary.org/wiki/Gell-Mann_Amnesia_effect">
                                "Gell-Mann amnesia"
                            </a>
                        </b>
                        ": If you're lucky enough to have an expertise, I'm sure you've noticed that "
                        {ai}
                        " is really good at most things except the thing you know a lot about."
                    </li>
                    <li>
                        <b>
                            <a href="https://en.wikipedia.org/wiki/Anthropomorphism">
                                "Anthropomorphism"
                            </a>
                        </b>
                        ": This is the big one for me. The used terminology in "
                        {ai}
                        " is built to uphold the illusion of intelligence."
                    </li>
                </ul>
            </section>
            <section>
                <h2>"Anthropomorphism"</h2>
                <ul class="lies">
                    <li>
                        "You don't "<em>"train"</em>" an "{ai}
                        " model. You optimize a model to be able to predict specific data. This interpretation would be "
                        <a href="https://openai.com/new-york-times/#ai-training-is-fair-use">
                            "legally problematic"
                        </a>"."
                    </li>
                    <li>
                        {llms}" don't "<em>"make mistakes"</em>
                        ". Mistakes require intention. For the same reason, they don't "
                        <em>"lie"</em>
                        ", but they are optimized to be able to recreate every single Sci-Fi story ever written that contains an "
                        {ai}". I don't really know which is worse."
                    </li>
                    <li>
                        {llms}" are not "<em>"intelligent"</em>
                        ". We all know this, even those that design them are open about this. The point where we don't agree is that they think that "
                        {llms}
                        " are so close to intelligence that they might somehow suddenly turn intelligent; you may have heard of this as \"the singularity\" or as \""
                        {agi}
                        "\", through the magic of \"recursive self-improvement\". That makes the water into wine routine look like child's play in comparison, and completely ignores the fact that \"recursive self-improvement\" is known as "
                        <a href="https://www.ibm.com/think/topics/model-collapse">
                            "model collapse"
                        </a>
                        " in the industry. Note that there is intelligence in text generated by "
                        {llms}", but that doesn't mean that the "{llm}
                        " is the source of that intelligence, no more than your fax machine is the source of the intelligence it prints."
                    </li>
                    <li>
                        "An "{llm}" is not "<em>"aware"</em>" that it's an "{ai}
                        ". It's a programmed response which (despite it literally claiming the opposite) makes it feel more human."
                    </li>
                    <li>
                        "You "<i>"could"</i>" argue that \"reasoning models\" are "
                        <em>"reasoning"</em>
                        ", but it's really more about creating a separate context with a more predictable path in the massive branching tree that is "
                        {llms}"."
                    </li>
                    <li>
                        {llms}" do "<b>"not"</b>" "<em>"hallucinate"</em>
                        ". The terminology suggests that this is a minor solvable issue, which ignores the truth; What they refer to as \"hallucination\" is in reality the only thing "
                        {llms}
                        " actually do. What's so cool about the technology is that the generated text often matches reality."
                    </li>
                </ul>
            </section>
            <section>
                <h2>"Why does it matter?"</h2>
                "My point with this clog post is that we all help perpetuate their illusion. Terminology and framing matters. If you think of "
                {llm}
                " models as lossy text compression, and not intelligence, it will help you understand what "
                {llms}
                " are capable of, and even what they will be capable of in the future. Most importantly, it will help you understand what it isn't; It's not your therapist; It's not intelligent; It's not self aware; It's not your friend; Any belief to the contrary is dangerous. We may some day create an actual "
                {ai}
                ", but it will not be an "
                {llm}
                ". You may rightfully argue that this is an oversimplification, but at least it's an oversimplification where you don't have to argue the philosophic point of "
                <quote>"well, what is intelligence, really?"</quote>
                "."
            </section>
        }
        .into_any()
        //<section>
        //    "There is a common saying: "<quote>"A bad workman blames his tools"</quote>.
        //    "As with many sayings, this is a misquote, this perticular one from the 13th century. The first known instance of the quote was "<quote cite="https://www.oxfordreference.com/display/10.1093/acref/9780199539536.001.0001/acref-9780199539536-e-83">a bad workman will never find a good tool</quote>" (as translated from "<quote lang="fr" cite="https://www.oxfordreference.com/display/10.1093/acref/9780199539536.001.0001/acref-9780199539536-e-83">"mauvés ovriers ne trovera ja bon hostill"</quote>")."
        //</section>
    }
}
