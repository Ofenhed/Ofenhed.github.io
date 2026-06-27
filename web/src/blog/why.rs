use crate::{
    blog::metadata::{BlogEntry, Locale, Tag},
    helpers::Abbr,
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

#[derive(Clone, Copy)]
pub(crate) struct WhyBlog;

impl BlogEntry for WhyBlog {
    fn uid() -> u32 {
        1
    }

    fn publish_date() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-06-22T12:00:00+01:00")
            .unwrap()
            .into()
    }

    fn publish() -> bool {
        true
    }

    fn locale() -> Option<Locale> {
        Locale::EnglishSimplified.into()
    }

    fn title() -> &'static str {
        "Another voice on the internet"
    }

    fn tags() -> &'static [Tag] {
        &[]
    }

    fn pin() -> Option<usize> {
        Some(1)
    }
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
