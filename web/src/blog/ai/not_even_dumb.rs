use crate::{
    blog::metadata::{BlogEntry, Locale, Tag, date},
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

    const TITLE: &'static str = "LLMs are not even dumb";

    const TAGS: &'static [Tag] = &[Tag::Ai, Tag::Tech];
}

#[lazy_route]
impl LazyRoute for NotEvenDumb {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        view! {
            <div class:videos class:carousel-or-grid>
                <YouTube video=youtube!("3fYiLXVfPa4" (9:16)) />
                <YouTube video=youtube!("gPthZLTnzu8" (9:16)) />
                <YouTube video=youtube!("bsl46vGpMNU" (9:16)) />
                <YouTube video=youtube!("7lRbNbwuczQ" (9:16)) />
                <YouTube video=youtube!("oltt1pIYtyY" (9:16)) />
                <YouTube video=youtube!("fcp1m-A-QwM" (9:16)) />
            </div>
        }
        .into_any()
    }
}
