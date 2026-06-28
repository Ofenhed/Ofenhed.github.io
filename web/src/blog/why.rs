use crate::{
    blog::metadata::{BlogEntry, Locale, Tag, date},
    helpers::Abbr,
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct WhyBlog;

impl BlogEntry for WhyBlog {
    const UID: u32 = 1;

    const PUBLISH_DATE: DateTime<Utc> = date(2026, 6, 22);

    const LOCALE: Option<Locale> = Some(Locale::EnglishSimplified);

    const TITLE: &'static str = "Another voice on the internet";

    const TAGS: &'static [Tag] = &[];

    const PIN: Option<usize> = Some(1);
}

#[lazy_route]
impl LazyRoute for WhyBlog {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let llm = || view! { <Abbr title="Large Language Model">LLM</Abbr> }.into_inner();
        view! {
            "This is an experiment with this new concept I thought of, where someone could share their thought in some kind of captain's log on the web, like web captain's log, or a captain's web log. With all "
            {llm}
            " generated text online, we clearly need more people sending text into the dark void that is the modern internet."
        }.into_any()
    }
}
