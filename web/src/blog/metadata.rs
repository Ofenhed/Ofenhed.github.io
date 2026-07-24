use std::{borrow::Cow, marker::PhantomData, pin::Pin};

use chrono::{DateTime, Utc};
use futures::FutureExt as _;
use leptos_router::{LazyRoute, PartialPathMatch, PathSegment, PossibleRouteMatch};
use strum::{EnumString, IntoStaticStr, VariantArray};

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Tag {
    Ai,
    Integrity,
    Tech,
    Review,
    Keyboards,
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Locale {
    #[strum(serialize = "sv_SE", serialize = "sv")]
    Swedish,
    #[strum(serialize = "en_US", serialize = "en")]
    EnglishSimplified,
}

impl PossibleRouteMatch for Locale {
    fn optional(&self) -> bool {
        false
    }
    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let mut param_offset = 0;
        let mut param_len = 0;
        let mut test = path.chars();

        if let Some('/') = test.next() {
            param_offset = 1;
        }

        let locale = match (test.next(), test.next()) {
            (Some('s'), Some('v')) => Some(Locale::Swedish),
            (Some('e'), Some('n')) => Some(Locale::EnglishSimplified),
            _ => None,
        };

        if locale.is_some() {
            param_len += 2;
        }

        match locale {
            Some(locale) if locale == *self => Some(PartialPathMatch::new(
                &path[param_offset + param_len..],
                vec![],
                &path[param_offset..param_offset + param_len],
            )),
            _ => None,
        }
    }
    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        path.push(PathSegment::Static(Cow::Borrowed(match self {
            Locale::Swedish => "sv",
            Locale::EnglishSimplified => "en",
        })))
    }
}

pub const fn date(year: i32, month: u32, day: u32) -> DateTime<Utc> {
    DateTime::from_naive_utc_and_offset(
        chrono::NaiveDateTime::new(
            chrono::NaiveDate::from_ymd_opt(year, month, day).expect("No user controlled input"),
            chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
        ),
        Utc,
    )
}

pub trait BlogEntry: LazyRoute + Clone + Sync {
    const UID: u32;
    const PUBLISH_DATE: DateTime<Utc>;
    const TITLE: &'static str;
    const AUTHOR: &'static str = crate::AUTHOR;
    const TAGS: &'static [Tag];

    const LOCALE: Option<Locale> = None;

    const PATH_LOCALE: bool = false;
    const HIDDEN: bool = false;

    const LAST_UPDATED: Option<DateTime<Utc>> = None;

    const PIN: Option<usize> = None;
}

pub trait BlogEntryHandler {
    type Result;
    fn with_blog<B: BlogEntry>(&mut self, blog: B) -> Self::Result;
}

#[derive(Clone, Copy)]
pub struct BlogEntryHandlerFor<T>(PhantomData<T>)
where
    Self: BlogEntryHandler<Result = T>;

impl<T> BlogEntryHandlerFor<T>
where
    Self: BlogEntryHandler<Result = T>,
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

pub fn with_blog_simple<T>(blog: impl BlogEntry) -> T
where
    BlogEntryHandlerFor<T>: BlogEntryHandler<Result = T>,
{
    BlogEntryHandlerFor::<T>::new().with_blog(blog)
}

impl BlogEntryHandler for () {
    type Result = ();
    fn with_blog<B: BlogEntry>(&mut self, _blog: B) -> Self::Result {}
}

pub struct PreloadUids(pub Vec<u32>);

impl BlogEntryHandler for PreloadUids {
    type Result = Option<Pin<Box<dyn Future<Output = ()>>>>;

    fn with_blog<B: BlogEntry>(&mut self, _blog: B) -> Self::Result {
        self.0
            .iter()
            .enumerate()
            .find_map(|(idx, uid)| if *uid == B::UID { Some(idx) } else { None })
            .map(|idx| {
                self.0.swap_remove(idx);
                B::preload().boxed_local()
            })
    }
}
