use crate::{
    blog::{
        ai::not_even_dumb::NotEvenDumb,
        blog_entry_href,
        metadata::{BlogEntry, Locale, Tag, date},
    },
    helpers::{Abbr, BoxType, Footnote, Url, idle_preload},
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct WhatAreLLMs;

impl BlogEntry for WhatAreLLMs {
    const UID: u32 = 4;

    const PUBLISH_DATE: DateTime<Utc> = date(2026, 6, 6);

    const LAST_UPDATED: Option<DateTime<Utc>> = Some(date(2026, 8, 14));

    const LOCALE: Option<Locale> = Some(Locale::EnglishSimplified);

    const TITLE: &'static str = "An intuitive understanding of LLMs";

    const TAGS: &'static [Tag] = &[Tag::Ai, Tag::Tech];

    fn title() -> Option<AnyView> {
        Some(
            view! {
                "An intuitive understanding of "
                <Abbr title="Large Language Model" suffix="s" no_expand=true>
                    "LLM"
                </Abbr>
            }
            .into_any(),
        )
    }
}

#[lazy_route]
impl LazyRoute for WhatAreLLMs {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        idle_preload::<NotEvenDumb>();
        let not_even_dumb_link = || blog_entry_href::<NotEvenDumb>();
        let ai = || {
            view! {
                <Abbr no_expand=true title="Artificial Intelligence">
                    "AI"
                </Abbr>
            }
            .into_inner()
        };
        let jpeg = || {
            view! {
                <Abbr no_expand=true title="Joint Photographic Experts Group">
                    JPEG
                </Abbr>
            }
            .into_inner()
        };
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
        let asi = || view! { <Abbr title="Artificial Super Intelligence">ASI</Abbr> }.into_inner();
        view! {
            <p>
                "When we talk about " <q>{ai}</q> " today, we pretty much talk about " {llms}
                ". We're being told that they are able to (or very soon able to) fully replace some human workers. I'm writing this to help put some context to that statement. I'm trying to be concise; my priority with this text is for it to be accessible."

                "To be able to understand what an " {llm}
                " is, we'll take a detour to compression algorithms."
            </p>
            <span {..BoxType::ExtraInfo
                .attr()}>
                "There are two categories of compression: lossless compression and lossy compression."
                <dl>
                    <dt>"Lossless compression"</dt>
                    <dd>
                        "This will always get back the exact data you had before compression. You're using this right now, in your browser. It's most often done by applying known predefined algorithms designed to find and efficiently describe patterns. The compression ratio is decided by the effectiveness of the algorithm and how much computing power and memory you provide."
                    </dd>
                    <dt>"Lossy compression"</dt>
                    <dd>
                        "In this case you lose data when you compress. This is often done by looking for things that can be removed without us noticing, such as high pitch sounds and dark colors, in combination with approximations of patterns. The compression ratio here is decided by the same as lossless compression, but also by how much quality degradation you can accept. This is what's used in most photographs and videos, and is what's responsible for "
                        {jpeg} " images sometimes being blocky."
                    </dd>
                </dl>
            </span>
            <p>
                "Historically, text has almost exclusively been used with lossless compression. "
                {llms} " are a revolutionary change to that. " {llms}
                " are not compression in the traditional sense, where you'd expect the compressed file to grow when you give it more data, compressed by a predefined set of algorithms. With "
                {llms}
                " you create the output file from the start, then you take an arbitrary amount of data and optimize the parameters in that existing output file to be able to approximate the data you provide."
            </p>
            <p>
                "I get that some of you will have an instant reaction of this comparison being ridiculous. I would like to direct you to a study"
                <Footnote id=Oco::Borrowed("extracting-books-from-llm")>
                    <b>Extracting books from production language models</b>
                    <br />
                    <i>"Ahmed Ahmed, A. Feder Cooper, Sanmi Koyejo, Percy Liang"</i>
                    <br />
                    <Url>"https://arxiv.org/pdf/2601.02671v1"</Url>
                </Footnote> " where researchers could get " {llm}
                " models to recall (near-verbatim) 95.8% of the first Harry Potter book and 95.5% of 1984. This doesn't prove lack of intelligence, but it proves that the data is there."
            </p>

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
                    <b>"Complexity"</b>
                    ": It's difficult (probably not possible) to imagine the amount of data that has been used to optimize these models, or the amount of parameters that a model uses. It's a lot easier to just assume that there is some kind of intelligence."
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
                    <b>"Generous interpretation"</b>
                    ": If you read the sentence "
                    <q>"c u in 5 at bus stop"</q>
                    ", you will likely assume what the author meant. We are good at this, and it usually serves us well as there's usually an intent behind words. When there isn't, we're still likely to fill that gap, and then assume that the meaning we found was intended."
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
                    <Footnote id=Oco::Borrowed(
                        "ai-warning-trojan",
                    )>
                        {ai}" companies keep telling us about how dangerous "{ai}
                        " can be, they even ask for new laws. Anthropic claims that "
                        <a href="https://www.ynetnews.com/tech-and-digital/article/hkftl9ibmg">
                            "Claude Mythos "{llm}" is too dangerous for the public"
                        </a>
                        ". They warn that the models might at any time start improving themselves without human help, become super intelligent, and take over the world. "
                        <em>
                            "All of these claims strengthens the confidence in "{llm}
                            " as a road to intelligence. It also keeps lawmakers focused on sci-fi threats, instead of the damage the "
                            {ai}" hype is already responsible for."
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
                    {llms}
                    " are really good at most things except the thing you know a lot about."
                </li>
                <li>
                    <b>
                        <a href="https://en.wikipedia.org/wiki/Anthropomorphism">
                            "Anthropomorphism"
                        </a>
                    </b>
                    ": This is the big one for me. The used terminology around "
                    <q>{ai}</q>
                    " is built to uphold the illusion of intelligence."
                </li>
            </ul>
            "As you may have noticed, this list is kind of problematic in the sense where it quickly deteriorated from sources of intelligence to pillars upholding the illusion of intelligence."
            <h2>"Anthropomorphism"</h2>
            <blockquote cite="Oxford English Dictionary, 1st ed. \"anthropomorphism, n.\" Oxford University Press (Oxford), 1885">
                "Anthropomorphism is the ascribing of human personality, appearance, conduct, cognition, or other attributes to non-human entities, often including non-human animals."
            </blockquote>
            <p>
                "Let's break down some of this anthropomorphism, and why I find it problematic. I have skipped some obvious ones, such as calling them "
                <q title="This word means something completely different to people outside of computer science">
                    {ai} " agents"
                </q>
                " or voice models breathing and laughing, and focused on the ones that are a bit more subtle."
            </p>
            <ul class:pink-marker=true>
                <li>
                    "You don't "<mark>"train"</mark>" an "{llm}
                    " model. You optimize a model to be able to generate an approximation of specific data. This interpretation would be "
                    <a href="https://openai.com/new-york-times/#ai-training-is-fair-use">
                        "legally problematic"
                    </a>"."
                </li>
                <li>
                    {llms}" don't "<mark>"make mistakes"</mark>
                    ". Mistakes require intention. For the same reason, they don't "
                    <mark>"lie"</mark>
                    ", but they are optimized to be able to recreate every single Sci-Fi story ever written that contains an "
                    {ai}". I don't really know which is worse."
                </li>
                <li>
                    {llms}" are not "<mark>"intelligent"</mark>
                    ". Most know this on some level, and even those that design them are somewhat open about this, albeit not in such clear phrasing. The point where we don't agree is that they think that "
                    {llms}
                    " are so close to intelligence that they might somehow suddenly turn intelligent; you may have heard of this as "
                    <q>"the singularity"</q>", " {agi} ", or "{asi} ", through the magic of "
                    <q>"recursive self-improvement"</q>
                    ". This is akin to a stage magician suddenly being able to perform real telekinesis, and any description on how "
                    <q>"recursive self-improvement"</q>
                    " would actually work sounds like what's already known as "
                    <a href="https://www.ibm.com/think/topics/model-collapse">"model collapse"</a>
                    " in the industry. The problem here is that even if you know that " {llms}
                    " aren't intelligent, it's really easy to get tricked into doubt. Note that there is intelligence in text generated by "
                    {llms}", but that doesn't mean that the "{llm}
                    " is the source of that intelligence, no more than your fax machine is the source of the intelligence it prints. Interestingly, on the same note, they aren't even "
                    <mark>"stupid"</mark>"; they're something else"
                    <Footnote id=Oco::Borrowed(
                        "not-even-dumb",
                    )>
                        "I've collected " <a href=not_even_dumb_link>"a couple of videos"</a>
                        " to demonstrate what I mean by this, but I would recommend reading the rest of this clog post first."
                    </Footnote>"."
                </li>
                <li>
                    "You "<i>"could"</i>" argue that "<q>"reasoning models"</q>" are "
                    <mark>"reasoning"</mark>
                    ", but it's really more about creating a separate context with a more predictable path in the massive branching tree that is "
                    {llms}"."
                </li>
                <li>
                    {llms}" do "<b>"not"</b>" "<mark>"hallucinate"</mark> ". "
                    <b>"This one is really important"</b>
                    ". The terminology suggests that this is a minor solvable issue, a quirk of the current stage of "
                    {llms}", which ignores the truth; What they refer to as " <q>"hallucination"</q>
                    " is in reality the "<i>"only"</i>" thing " {llms}
                    " actually do. What's so cool about the technology is that the generated text often matches reality."
                </li>
            </ul>

            <h2>"Why does it matter?"</h2>
            <p>
                "My point with this clog post is that we all perpetuate the illusion of intelligence. Terminology and framing matters. Thinking of "
                {llms} " as " {ai}
                " creates a abstract entity with hard to define properties, and it becomes very easy to fall into the anthropomorphism trap. On the other hand, if you think of "
                {llm} " models as lossy text compression, it will help you understand what " {llms}
                " are actually capable of, and even somewhat what they will be capable of in the future. Most importantly, it will help you understand what it isn't; It's not your therapist; It's not intelligent; It's not self aware; It's not your friend; Any belief to the contrary is dangerous. We may some day create an actual "
                {ai} ", but it will not be an " {llm}
                ". You may rightfully argue that this is an oversimplification, but at least it's an oversimplification where you don't have to argue the philosophic point of "
                <q>"well, what is intelligence, really?"</q> "."
            </p>
            <p>
                "While I would like to believe that "{llms}
                " are without value, that's not what I'm saying. I don't agree that they are important enough to displace global environmental sustainability, but they are useful. That said, when you hype the technology to people in power that don't understand what the technology really is, they're going to make sure to promote it to a "
                <a href="https://en.wikipedia.org/wiki/Peter_principle">
                    "level of respective incompetence"
                </a>". I am very confident in my belief that "{llms}
                " won't have an uprising and hack nuclear missile facilities, but my confidence in the leaders of the world to not try to pigeonhole "
                {llms}" into surveillance or weapon systems is a lot lower."
            </p>
        }
        .into_any()
        //<section>
        //    "There is a common saying: "<q>"A bad workman blames his tools"</q>.
        //    "As with many sayings, this is a misquote, this particular one from the 13th century. The first known instance of the quote was "<q cite="https://www.oxfordreference.com/display/10.1093/acref/9780199539536.001.0001/acref-9780199539536-e-83">a bad workman will never find a good tool</q>" (as translated from "<q lang="fr" cite="https://www.oxfordreference.com/display/10.1093/acref/9780199539536.001.0001/acref-9780199539536-e-83">"mauvés ovriers ne trovera ja bon hostill"</q>")."
        //</section>
    }
}
