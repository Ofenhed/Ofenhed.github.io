use std::borrow::Cow;

use crate::{
    blog::{
        emails::ChatControlReplyV,
        metadata::{BlogEntry, Locale, Tag, date},
    },
    helpers::{Abbr, Footnote, Url},
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct ChatControl;

impl BlogEntry for ChatControl {
    const UID: u32 = 5;

    const PUBLISH_DATE: DateTime<Utc> = date(2025, 8, 27);

    const LOCALE: Option<Locale> = Some(Locale::EnglishSimplified);

    const TITLE: &'static str = "In opposition of ChatControl (Open letter)";

    const TAGS: &'static [Tag] = &[Tag::Ai, Tag::Tech, Tag::Integrity];

    const PIN: Option<usize> = None;
}

#[lazy_route]
impl LazyRoute for ChatControl {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let osint = || view! { <Abbr title="Open Source Intelligence">"OSINT"</Abbr> }.into_inner();
        view! {
            <fieldset>
                <legend>Note</legend>
                "This is a letter I sent to my representatives on "
                <time datetime="2025-08-27">"the 27th of August, 2025"</time>
                ". To date, the only reply I've received has been from "
                <a href=format!("/clog/{}", ChatControlReplyV::UID)>"Vänsterpartiet"</a>
                ", who also opposes Chat Control."
            </fieldset>
            <p>"Hello,"</p>
            <section>
                <p>
                    "I am a developer and an "
                    <b>"IT security specialist with over a decade in the industry"</b>
                    ". A significant portion of this time has been in the role of a penetration tester, which is a role in IT security where "
                    <b>"my task is to find and exploit security vulnerabilities"</b>
                    ", to enable my customers to address the issues I find and improve their security. My work as a developer has also centered around IT security, 3 years of which was spent "
                    <b>"working on encrypted communication solutions for military applications"</b>
                    ". Apart from my professional experience in the field, cryptography is a big personal interest of mine. Speaking of personal interest, I'm also a father wanting to protect my children. This letter is "
                    <b>
                        "my own personal and professional opinions without any usage of AI or online templates"
                    </b>"."
                </p>
                <p>
                    "I'm writing in the " <b>"strongest possible opposition of chat control"</b>"."
                </p>
            </section>
            <p>
                "In December of 2024, the American FBI urged users to stop sending SMS"
                <Footnote id="stop-sending-sms">
                    <Url>
                        "https://www.forbes.com/sites/zakdoffman/2024/12/06/fbi-warns-iphone-and-android-users-stop-sending-texts/"
                    </Url>
                </Footnote>
                ", because they had reason to believe that Chinese adversaries had gained access to the US phone infrastructure. In 2019, The Guardian reported that Poland had arrested Huawei employees on espionage charges"
                <Footnote id="huawei-spy-in-poland">
                    <Url>
                        "https://www.theguardian.com/technology/2019/jan/11/huawei-employee-arrested-in-poland-over-chinese-spy-allegations"
                    </Url>
                </Footnote>
                ". In Sweden we've blocked Huawei from participating in the Swedish 5G network"
                <Footnote id="huawei-5g">
                    <Url>
                        "https://www.domstol.se/nyheter/2022/06/forbud-mot-produkter-fran-huawei-star-fast/"
                    </Url>
                </Footnote> ", because of (very legitimate) security concerns. "
                <b>
                    "We have very real adversaries with massive resources trying to gain access to our communications"
                </b>
                ". This is one of many good reasons to use strong encryption. Trying to weaken or circumvent this encryption also opens the door to these adversaries."
            </p>
            <p>
                "Proponents of this law are claiming that this protection can be added without breaking the security. This is a "
                <b>"completely false claim"</b>". There are no equivalents to "<q>"sniffer dogs"</q>
                " for encryption. Looking at the encrypted message you can deduce an approximate size of the original package, intended receiver of the package, in some cases the sender of the package, and the time and quantities of messages. That's it. Anything more than this and you will have to break encryption. Even if we assume that the technology works as intended, as in only collecting suspicious material, users will never know if anything they send will also be secretly sent to an unknown third party. "
                <b>"This inherently leads to self censorship"</b>
                ", especially in countries with growing antidemocratic forces, and for a very valid reason. For historical precedence look at Netherlands. In the 1930s, they had records that allowed their tax collectors to collect taxes for religious institutions. This was great for churches and their members alike. It was also great for the Nazis, leading to 70% of the Jewish population being murdered. It's very hard to know today what will be a secret in 10 years."
            </p>
            <p>
                "The suggested solution on how to do this "
                <q class="para">"without compromising security"</q>
                " is to scan messages for prohibited material before the messages are encrypted. This means that you have one of two solutions. Either you make every provider of chat services implement this solution by themselves, or you provide them with a solution from a third party. Alternative 1 forces them to become experts in an extremely complex technology far outside of their primary business, where mistakes may completely break their users privacy and overload law enforcement. This pretty much means that you will have to provide the solution from a third party."
            </p>
            <section>
                <p>"There are two main techniques that can be used:"</p>
                <ol>
                    <li>
                        "The black list, where images sent are compared to known thumbprints of prohibited images, which are not sensitive to recompression artefacts. This is technology that Apple, one of the most advanced and financially capable companies in the world, has attempted and failed to implement for the last couple of years."
                    </li>
                    <li>
                        "Heuristic analysis, where AI is an example. This introduces a ridiculous amount of complexity and trust. The reason for this is that AI models are huge sets of opaque variables. These models can be (very shallowly) tested, but "
                        <b>"they cannot be reviewed, and they are notoriously hard to control"</b>
                        ". An AI model can easily be trained to increase trigger rate for users with undesired "
                        <b>"political opinions"</b>
                        " measured over thousands of messages in a way that will never be caught in a test."
                    </li>
                </ol>
            </section>
            <p>
                "Whoever is tasked with developing this technology is granted the ability to run code that will scan pretty much every single message sent inside of the European Union. "
                <b>"If developers get corrupted, they can covertly"</b>
                " convert the analysis tool into a tool that "
                <b>
                    "spies and collects information on the entire European union in a way that has never been seen before"
                </b>
                ". While the intention is to only send back information when abuse material is is found, the information sent back can easily be encoded to include hidden information with steganographic methods. They gain the ability to collect opinions, political leanings, secrets, insecurities, and fears of the entire European population. That is "
                <b>"information that in the wrong hands can completely topple democracy"</b>
                ". How far do you think a foreign adversary would go to gain access to this information? This ability in combination with the low transparency of AI models (thus low detectability of malicious applications) means that anyone tasked with developing this technology will be put at significant personal risk, as will their families."
            </p>
            <p>
                "You add exclusions for "<q>"national security purposes"</q>
                ", what does that even mean? What about the phones of their partners, children, and friends? In "
                {osint}
                " there is the concept of aggregated data, meaning that you combine incomplete data from multiple sources to create a bigger picture. An AI model can be trained to specifically to target people with proximity to a target, and any data collected can be excused as "
                <q>"false positives that will be ignored."</q>" This technology would be "
                <b>"a threat even to the stated exclusions in the proposals"</b>
                ". Besides the issue with data collection, this also risks actually good tools used both by Swedish companies and your own military to communicate, such as Signal, simply being lost"
                <Footnote id=Cow::Borrowed("signal-leaving-eu")>
                    <blockquote cite="https://signal.org/blog/pdfs/germany-chat-control.pdf">
                        "If we were given a choice between building a surveillance machine into Signal or leaving the market, we would leave the market."
                        <cite>
                            <Url>"https://signal.org/blog/pdfs/germany-chat-control.pdf"</Url>
                        </cite>
                    </blockquote>
                </Footnote>". "
                <b>"This would introduce a meaningful risk to companies and our military alike"</b>
                "."
            </p>
            <p>
                "The proponents want to make you feel as if you either support Chat Control or you enable child sexual abuse. This is "
                <b>"manipulation, known as the false dilemma or false dichotomy"</b>
                ". I'm not saying it won't catch child abusers, I really can't predict that, but I'm saying that you're given the apparent option of catching or not catching child abusers. Another way of catching child abusers would be to arrest and investigate every single grown up working with "
                <b>"konfirmationsläger"</b> <Footnote id="other-illegitimate-targets">
                    <Url>
                        "https://www.kyrkanstidning.se/nyhet/ung-ledare-domd-sexuellt-ofredande-pa-lager"
                    </Url>
                    <br />
                    <Url>
                        "https://www.expressen.se/nyheter/ledare-ofredade-14-aring-pa-konfirmationslager/"
                    </Url>
                    <br />
                    <Url>"https://yle.fi/a/7-10061536"</Url>
                    <br />
                    <Url>
                        "https://www.smt.se/2025-05-27/ledare-anklagades-for-sexofredande-pa-konfalager/"
                    </Url>
                    <br />
                    <Url>
                        "https://www.svt.se/nyheter/lokalt/varmland/sexovergrepp-pa-konfirmationslager"
                    </Url>
                    <br />
                    <Url>
                        "https://www.dagensjuridik.se/nyheter/barnvaldtaktsdomd-konfirmationsledare-frias-i-hovratten-inte-fraga-om-uppenbart-overgrepp/"
                    </Url>
                    <br />
                    <Url>"https://www.svenskakyrkan.se/motsexuellaovergrepp"</Url>
                </Footnote>
                ". Very few of them would actually get charged, but you would catch child abusers that directly hurt children. If you don't do this, does that mean that you are opposed to catching child abusers, or simply that you have respect for individual legal rights? I've seen the statistics, I know that the "
                <b>"numbers are really bad for the EU"</b>
                ", so I recognize that you're desperate for a solution. Chat Control is not that answer. It's a Hail Mary grounded in a very low technical understanding of cryptography and that very same desperation."
            </p>
            <section>
                <p>
                    "If the intent of this proposal is to find child abusers, then I think it's a very over engineered proposal with "
                    <b>"huge ramifications for citizens and businesses alike"</b>
                    ", financial costs, and likely underwhelming results. On the other hand, "
                    <b>
                        "if the intent is to actually map and track every single European citizen"
                    </b>
                    ", then this is a very efficient (albeit also very risky) proposal. This leads me to to the conclusion that at least "
                    <b>"one of following"</b>" is true for anyone supporting the suggestion:"
                </p>
                <ul>
                    <li>
                        "You don't understand neither the proposed implementation or its feasibility, nor its implications"
                    </li>
                    <li>
                        "You have been manipulated into accepting the false dilemma that you have to support either Chat Control or child abusers"
                    </li>
                    <li>
                        "You think that with the EU numbers being so bad, a terrible solution is better than no solution"
                    </li>
                    <li>"You're a performative ally"</li>
                    <li>"You have actual malicious intent to spy on the European population"</li>
                </ul>
            </section>
            <p>
                "Beyond the very questionable legality around this proposal (which other people cover way better than I possibly could), I do think it is an actual huge threat to democracy, "
                <b>"even if implemented perfectly"</b>
                ". While I find the first three reasons way more likely than the last two options, I have yet to find a reason to support this proposal that does not erode trust in politicians. As my representative in the European Union I urge you to "
                <b>"please oppose this proposal or enlighten me if you think I'm wrong"</b>
                " in my conclusion."
            </p>
            <p lang="sv">"Med vänliga hälsningar,"<br /> "Marcus Ofenhed"</p>
        }.into_any()
    }
}
