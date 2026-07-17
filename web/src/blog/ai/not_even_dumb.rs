use crate::{
    blog::metadata::{BlogEntry, Locale, Tag, date},
    helpers::Abbr,
    third_party::{YouTube, youtube},
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct NotEvenDumb;

impl BlogEntry for NotEvenDumb {
    const UID: u32 = 7;

    const PUBLISH_DATE: DateTime<Utc> = date(2026, 7, 13);
    const HIDDEN: bool = true;

    const LOCALE: Option<Locale> = Some(Locale::EnglishSimplified);

    const TITLE: &'static str = "LLMs are not even stupid";

    const TAGS: &'static [Tag] = &[Tag::Ai, Tag::Tech];
}

#[lazy_route]
impl LazyRoute for NotEvenDumb {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let ai = || {
            view! {
                <Abbr no_expand=true title="Artificial Intelligence">
                    "AI"
                </Abbr>
            }
            .into_inner()
        };
        let llms = || {
            view! {
                <Abbr title="Large Language Model" suffix="s">
                    "LLM"
                </Abbr>
            }
            .into_inner()
        };
        view! {
            <a href="https://www.youtube.com/@FatherPhi">"FatherPhi"</a>
            " is a great YouTube channel collecting examples of weird interactions with "
            {ai}
            ". I've cherry-picked a collection of videos that I feel demonstrate how "
            {llms}
            " aren't stupid, they're something else entirely."
            <h2>"General intelligence"</h2>
            <div class:videos class:carousel-or-grid>
                <YouTube video=youtube!("3fYiLXVfPa4" (9:16)) />
                <YouTube video=youtube!("gPthZLTnzu8" (9:16)) />
                <YouTube video=youtube!("bsl46vGpMNU" (9:16)) />
                <YouTube video=youtube!("fcp1m-A-QwM" (9:16)) />
            </div>
            <h2>"Spelling issues"</h2>
            "These are somewhat dismissable as the model using tokens instead of words."
            <div class:videos class:carousel-or-grid>
                <YouTube video=youtube!("7lRbNbwuczQ" (9:16)) />
                <YouTube video=youtube!("oltt1pIYtyY" (9:16)) />
                <YouTube video=youtube!("m_shkDLGWEQ" (9:16)) />
            </div>
            <h2>"Vision issues"</h2>
            <div class:videos class:carousel-or-grid>
                <YouTube video=youtube!("pBLvzATPxv8" (9:16)) />
            </div>
        }
        .into_any()
    }
}
