use std::{iter, marker::PhantomData, pin::Pin};

use chrono::{DateTime, Utc};
use futures::FutureExt as _;
use leptos_router::{LazyRoute, MatchNestedRoutes};
use strum::{AsRefStr, EnumString, VariantArray};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Tag {
    Ai,
    Tech,
    Review,
    Keyboards,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum Locale {
    #[strum(serialize = "sv_SE")]
    Swedish,
    #[strum(serialize = "en_UK")]
    English,
    #[strum(serialize = "en_US")]
    EnglishSimplified,
}

pub trait BlogEntry: Sized + LazyRoute + Clone + Sync + 'static {
    fn uid() -> u32;
    fn publish_date() -> DateTime<Utc>;
    fn title() -> &'static str;
    fn tags() -> &'static [Tag];

    fn publish() -> bool {
        false
    }

    fn locale() -> Option<Locale> {
        None
    }

    fn last_updated() -> Option<DateTime<Utc>> {
        None
    }

    fn pin() -> Option<usize> {
        None
    }
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
            .find_map(|(idx, uid)| if *uid == B::uid() { Some(idx) } else { None })
            .map(|idx| {
                self.0.swap_remove(idx);
                B::preload().boxed_local()
            })
    }
}

#[derive(Clone)]
pub struct NoMatch;
impl MatchNestedRoutes for NoMatch {
    type Data = ();

    type Match = ();

    fn match_nested<'a>(
        &'a self,
        path: &'a str,
    ) -> (Option<(leptos_router::RouteMatchId, Self::Match)>, &'a str) {
        (None, path)
    }

    fn generate_routes(&self) -> impl IntoIterator<Item = leptos_router::GeneratedRouteData> + '_ {
        iter::empty()
    }

    fn optional(&self) -> bool {
        todo!()
    }
}
