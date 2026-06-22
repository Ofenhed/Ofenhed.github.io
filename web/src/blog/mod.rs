pub mod ai;
pub mod metadata;
pub mod unremarkable;
pub mod why;

use crate::{
    app::HamburgerMenu,
    blog::{
        metadata::{
            BlogEntry, BlogEntryHandler, BlogEntryHandlerFor, Locale, NoMatch, PreloadUids, Tag,
            with_blog_simple,
        },
        unremarkable::Unremarkable,
    },
    helpers::{AddContext, ForRoute},
};
use chrono::{DateTime, Utc};
use leptos::{
    attr::custom::custom_attribute, either::Either, ev, prelude::*, task::spawn_local_scoped,
};
use leptos_meta::{Meta, use_head};
#[allow(unused)] // False positive
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    Lazy, PartialPathMatch, PathSegment, PossibleRouteMatch, SsrMode, StaticSegment,
    any_nested_route::{AnyNestedRoute, IntoAnyNestedRoute},
    components::{A, ParentRoute, Route},
    hooks::use_params,
    nested_router::Outlet,
    params::Params,
    path,
    static_routes::StaticRoute,
};
use std::{borrow::Cow, cmp::max, str::FromStr};
use strum::{AsRefStr, EnumString, VariantArray};

const ENTRIES_PER_PAGE: usize = 10;

pub fn with_blogs<B: BlogEntryHandler>(mut b: B) -> impl Iterator<Item = B::Result> {
    [
        b.with_blog(why::WhyBlog),
        b.with_blog(Unremarkable),
        b.with_blog(ai::WhatAreLLMs),
    ]
    .into_iter()
}

pub fn with_blogs_simple<B>()
-> impl Iterator<Item = <BlogEntryHandlerFor<B> as BlogEntryHandler>::Result>
where
    BlogEntryHandlerFor<B>: BlogEntryHandler<Result = B>,
{
    with_blogs(BlogEntryHandlerFor::<B>::new())
}

impl BlogEntryHandler for BlogEntryHandlerFor<AnyNestedRoute> {
    type Result = AnyNestedRoute;

    fn with_blog<B: BlogEntry>(&mut self, blog: B) -> Self::Result {
        let metadata = with_blog_simple::<BlogEntryMeta>(blog.clone());
        let b = Lazy::<B>::new();
        #[cfg(debug_assertions)]
        let publish = true;
        #[cfg(not(debug_assertions))]
        let publish = metadata.publish;
        if publish {
            view! {
                <ParentRoute
                    path=metadata
                    view=move || {
                        view! {
                            <BlogHeading entry=blog.clone() />
                            <Outlet />
                        }
                    }
                    ssr=SsrMode::OutOfOrder
                >
                    <Route path=path!("") view=b ssr=SsrMode::Static(StaticRoute::new()) />
                </ParentRoute>
            }
            .into_inner()
            .into_any_nested_route()
        } else {
            NoMatch.into_any_nested_route()
        }
    }
}

#[component(transparent)]
pub fn BlogPaging() -> impl MatchNestedRoutes + Clone + 'static {
    #[derive(Params, PartialEq)]
    struct Page {
        page: usize,
    }
    let page = || {
        let page = use_params::<Page>();
        Signal::derive(move || {
            CurrentPage(
                max(
                    page.with(|x| x.as_ref().expect("Params always set here").page),
                    1,
                ) - 1,
            )
        })
    };
    let num_pages = |entries| {
        let entry_count = entries;
        let pages_full = entry_count + ENTRIES_PER_PAGE - 1;
        let pages = pages_full / ENTRIES_PER_PAGE;
        pages
    };
    let maybe_ignore = {
        #[cfg(not(feature = "statics"))]
        {
            ()
        }
        #[cfg(feature = "statics")]
        {
            let leptos_static_files::NoGenerateStatic(ignore) =
                use_context().expect("This should be defined by static_files");
            let blogs = use_context::<Signal<FilteredEntities>>()
                .expect("Filtered entities are always defined here");
            let page = use_context::<ReadSignal<CurrentPage>>()
                .expect("Current page always available here");
            Signal::derive(move || {
                let CurrentPage(page) = page.get();
                let page = page + 1;
                let last_page = num_pages(blogs.with(|x| x.0.len()));
                if last_page < page {
                    ignore.set(true);
                }
            })
        }
    };
    view! {
        <ParentRoute path=path!("") view=Outlet ssr=SsrMode::OutOfOrder>
            <Route
                path=path!("/page/:page")
                view=move || {
                    let page = page().get();
                    view! {
                        <AddContext context=page />
                        {maybe_ignore.clone()}
                    }
                }
                ssr=SsrMode::Static(
                    StaticRoute::new()
                        .prerender_params(move || {
                            async move {
                                let max_pages = num_pages(with_blogs(()).count());
                                [
                                    (
                                        "page".to_string(),
                                        (1..max_pages)
                                            .map(|x| (x + 1).to_string())
                                            .collect::<Vec<_>>(),
                                    ),
                                ]
                                    .into_iter()
                                    .collect()
                            }
                        }),
                )
            />
            <Route
                path=path!("/")
                view=move || {
                    view! {
                        <AddContext context=CurrentPage(0) />
                        {maybe_ignore}
                    }
                }
                ssr=SsrMode::Static(StaticRoute::new())
            />
        </ParentRoute>
    }
    .into_inner()
}

#[component(transparent)]
pub fn BlogSorting() -> impl MatchNestedRoutes + Clone + 'static {
    view! {
        <ParentRoute path=path!("") view=Outlet ssr=SsrMode::OutOfOrder>
            <ParentRoute path=path!("/sort") view=Outlet ssr=SsrMode::OutOfOrder>
                <ForRoute
                    each=SortBy::VARIANTS.iter()
                    children=|key| {
                        view! {
                            <ParentRoute
                                path=(StaticSegment(key.as_ref()),)
                                view=move || view! { <AddContext context=key.to_owned() /> }
                                ssr=SsrMode::OutOfOrder
                            >
                                <ParentRoute
                                    path=path!("/invert")
                                    view=|| view! { <AddContext context=SortInvert(true) /> }
                                    ssr=SsrMode::OutOfOrder
                                >
                                    <BlogPaging />
                                </ParentRoute>
                                <ParentRoute
                                    path=path!("/")
                                    view=|| view! { <AddContext context=SortInvert(false) /> }
                                    ssr=SsrMode::OutOfOrder
                                >
                                    <BlogPaging />
                                </ParentRoute>
                            </ParentRoute>
                        }
                            .into_inner()
                    }
                />
            </ParentRoute>
            <ParentRoute
                path=path!("/")
                view=|| {
                    view! {
                        <AddContext context=SortInvert(false)>
                            <AddContext context=SortBy::Default />
                        </AddContext>
                    }
                }
                ssr=SsrMode::OutOfOrder
            >
                <BlogPaging />
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component(transparent)]
pub fn BlogTagFilter() -> impl MatchNestedRoutes + Clone + 'static {
    let no_tag_filter: Option<TagFilter> = None;
    view! {
        <ParentRoute path=path!("") view=Outlet ssr=SsrMode::OutOfOrder>
            <ParentRoute path=path!("/tag") view=Outlet ssr=SsrMode::OutOfOrder>
                <ForRoute
                    each=Tag::VARIANTS.iter()
                    children=|key| {
                        view! {
                            <ParentRoute
                                path=(StaticSegment(key.as_ref()),)
                                view=move || {
                                    view! { <AddContext context=Some(TagFilter(key.to_owned())) /> }
                                }
                                ssr=SsrMode::OutOfOrder
                            >
                                <BlogSorting />
                            </ParentRoute>
                        }
                            .into_inner()
                    }
                />
            </ParentRoute>
            <ParentRoute
                path=path!("/")
                view=move || view! { <AddContext context=no_tag_filter /> }
                ssr=SsrMode::OutOfOrder
            >
                <BlogSorting />
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component(transparent)]
pub fn BlogListing(
    #[prop(into)] blogs: Signal<Vec<BlogEntryMeta>>,
) -> impl MatchNestedRoutes + Clone {
    let blogs = {
        let (sort_by, writer) = signal(SortBy::PublishDate);
        provide_context(writer);
        let (inverted, writer) = signal(SortInvert(false));
        provide_context(writer);
        let (page, writer) = signal(CurrentPage(0));
        provide_context(page);
        provide_context(writer);
        let (tags, writer) = signal(None::<TagFilter>);
        provide_context(writer);
        let filtered_entities = Signal::derive(move || {
            FilteredEntities(blogs.with(|b| {
                let filter = tags.get();
                b.into_iter()
                    .filter(|x| {
                        #[cfg(not(debug_assertions))]
                        {
                            if !x.publish {
                                return false;
                            }
                        }
                        if let Some(TagFilter(filter)) = filter {
                            x.tags.contains(&filter)
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            }))
        });
        provide_context(filtered_entities);
        let entries = Signal::derive(move || {
            let SortInvert(invert_sort) = inverted.get();
            let sort_by = sort_by.get();
            let FilteredEntities(mut blogs) = filtered_entities.get();
            blogs.sort_unstable_by(|a, b| {
                let (a, b) = if invert_sort { (b, a) } else { (a, b) };
                match sort_by {
                    SortBy::Default => match (&a.pin, &b.pin) {
                        (x, y) if x == y => b.publish_date.partial_cmp(&a.publish_date).unwrap(),
                        (None, None) => unreachable!("None == None"),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (Some(x), Some(y)) => x.cmp(&y),
                    },
                    SortBy::Title => a.title.partial_cmp(b.title).unwrap(),
                    SortBy::PublishDate => b.publish_date.partial_cmp(&a.publish_date).unwrap(),
                    SortBy::ModifyDate => b
                        .last_updated
                        .unwrap_or(b.publish_date)
                        .partial_cmp(&a.last_updated.unwrap_or(a.publish_date))
                        .unwrap(),
                }
            });
            let CurrentPage(current) = page.get();
            let (chunks, tail) = blogs.as_chunks::<ENTRIES_PER_PAGE>();
            match chunks.len() {
                x if current == x => tail,
                x if current > x => &blogs[..],
                _ => &chunks[current],
            }
            .to_vec()
        });
        provide_context(Signal::derive(move || CurrentPageEntries(entries.get())));
        entries
    };
    view! {
        <ParentRoute
            path=path!("/")
            view=move || {
                Effect::new(move |_| {
                    let blogs = blogs.with(|x| x.iter().map(|x| x.uid).collect());
                    for b in with_blogs(PreloadUids(blogs)).filter_map(|x| x) {
                        spawn_local_scoped(async move { b.await });
                    }
                });
                view! {
                    <Outlet />
                    <BlogEntryList entries=blogs />
                }
            }
            ssr=SsrMode::OutOfOrder
        >
            <BlogTagFilter />
        </ParentRoute>
    }
    .into_inner()
}
#[component(transparent)]
pub fn Blog() -> impl MatchNestedRoutes + Clone {
    let blogs = with_blogs_simple::<AnyNestedRoute>().collect::<Vec<_>>();
    let blog_metadata = with_blogs_simple::<BlogEntryMeta>().collect::<Vec<_>>();
    view! {
        <ParentRoute path=path!("/clog") view=Outlet ssr=SsrMode::OutOfOrder>
            <ParentRoute path=path!("/entry") view=Outlet ssr=SsrMode::Static(StaticRoute::new())>
                <ForRoute each=blogs children=|b| b />
            </ParentRoute>
            <BlogListing blogs=blog_metadata />
        </ParentRoute>
    }
    .into_inner()
}

#[derive(PartialEq, Eq, Clone)]
#[cfg_attr(debug_assertions, derive(Debug))]
#[slot]
pub struct BlogEntryMeta {
    uid: u32,
    publish_date: DateTime<Utc>,
    last_updated: Option<DateTime<Utc>>,
    locale: Option<Locale>,
    publish: bool,
    title: &'static str,
    tags: &'static [Tag],
    pin: Option<usize>,
}

impl BlogEntryHandler for BlogEntryHandlerFor<BlogEntryMeta> {
    type Result = BlogEntryMeta;

    fn with_blog<B: BlogEntry>(&mut self, blog: B) -> Self::Result {
        blog.into()
    }
}

impl<T: metadata::BlogEntry> From<T> for BlogEntryMeta {
    fn from(_: T) -> Self {
        BlogEntryMeta {
            uid: T::uid(),
            publish_date: T::publish_date(),
            last_updated: T::last_updated(),
            locale: T::locale(),
            publish: T::publish(),
            title: T::title(),
            tags: T::tags(),
            pin: T::pin(),
        }
    }
}

impl PossibleRouteMatch for BlogEntryMeta {
    fn optional(&self) -> bool {
        false
    }
    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let mut matched_len = 0;
        let mut param_offset = 0;
        let mut param_len = 0;
        let mut test = path.chars();

        if let Some('/') = test.next() {
            matched_len += 1;
            param_offset = 1;
        }

        for char in test {
            if char.is_numeric() {
                matched_len += char.len_utf8();
                param_len += char.len_utf8();
            } else {
                break;
            }
        }

        if matched_len == param_offset {
            return None;
        }

        let matched_num = &path[param_offset..param_len + param_offset];
        match u32::from_str(matched_num) {
            Ok(id) if id == self.uid => Some(PartialPathMatch::new(
                &path[param_offset + param_len..],
                vec![],
                &path[param_offset..param_offset + param_len],
            )),
            _ => None,
        }
    }
    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        path.push(PathSegment::Static(Cow::Owned(self.uid.to_string())))
    }
}

#[component]
pub(crate) fn BlogHeading<B: BlogEntry>(entry: B) -> impl IntoView {
    use leptos_meta::Title;
    _ = entry;
    use_head();
    let last_update = B::last_updated().map(|x| {
        view! { <Meta property="og:modified_time" content=x.to_rfc3339() /> }
    });
    let locale = B::locale().map(|x| {
        view! { <Meta property="og:locale" content=x.as_ref().to_string() /> }
    });
    view! {
        <Title formatter=|title: String| format!("{title} - Captains Log") text=B::title() />
        {locale}
        <Meta property="og:title" content=B::title() />
        <Meta property="og:article:author" content="Marcus Ofenhed" />
        <For
            each=move || B::tags().iter()
            key=|x| x.to_owned()
            children=|tag| {
                view! { <Meta property="og:article:tag" content=tag.as_ref().to_string() /> }
            }
        />
        <Meta property="og:article:published_time" content=B::publish_date().to_rfc3339() />
        {last_update}
        <h1 id="pageHeader">{B::title()}</h1>
        <p>{B::publish_date().date_naive().to_string()}</p>
    }
}

fn to_title<'a>(input: impl Into<Oco<'a, str>>) -> Oco<'a, str> {
    let output = input.into();
    if !output.chars().any(|x| !x.is_alphanumeric()) {
        output
    } else {
        Oco::Owned(
            output
                .chars()
                .filter_map(|x| match x {
                    x if x.is_alphanumeric() => Some(x),
                    ' ' => Some('-'),
                    _bad => None,
                })
                .collect(),
        )
    }
}

#[derive(Clone)]
#[allow(unused)]
struct FilteredEntities(Vec<BlogEntryMeta>);

#[allow(unused)]
struct CurrentPageEntries(Vec<BlogEntryMeta>);

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum SortBy {
    #[default]
    Default,
    PublishDate,
    ModifyDate,
    Title,
}

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct SortInvert(bool);

#[derive(Clone, Copy, Debug)]
pub struct TagFilter(Tag);

#[derive(Clone, Copy)]
struct CurrentPage(usize);

impl AsRef<usize> for CurrentPage {
    fn as_ref(&self) -> &usize {
        &self.0
    }
}

impl FromStr for SortInvert {
    type Err = std::io::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "inv" | "invert" => Ok(Self(true)),
            "" => Ok(Self(false)),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                s.to_string(),
            )),
        }
    }
}

#[component]
pub fn BlogEntryList(#[prop(into)] entries: Signal<Vec<BlogEntryMeta>>) -> impl IntoView {
    let HamburgerMenu(toggle) = use_context().expect("Hamburger menu must have been defined here");
    let on_click = move |_: ev::MouseEvent| {
        toggle.get().map(|x| x.set_checked(false));
    };
    let article_pinned = |entry: &BlogEntryMeta| {
        entry
            .pin
            .map(|pin| Either::Right(custom_attribute("pinned", pin)))
            .unwrap_or(Either::Left(()))
    };
    let article_unpublished = |entry: &BlogEntryMeta| {
        if entry.publish {
            Either::Right(())
        } else {
            Either::Left(custom_attribute("unpublished", "unpublished"))
        }
    };
    view! {
        <ul id="blog-entries">
            <For each=move || entries.get() key=|x: &BlogEntryMeta| x.uid let(entry)>
                <li {..article_pinned(&entry)} {..article_unpublished(&entry)}>
                    <article>
                        <A
                            on:click=on_click
                            href=move || {
                                format!("/clog/entry/{}#{}", entry.uid, to_title(entry.title))
                            }
                        >
                            {entry.title.to_owned()}
                        </A>
                        <time datetime=entry.publish_date.date_naive().to_string() />
                        <ul class="tags">
                            <For each=move || entry.tags key=|x| x.to_owned() let(tag)>
                                <li>
                                    <A href=move || {
                                        format!("/clog/tag/{}", tag.as_ref())
                                    }>{tag.as_ref().to_string()}</A>
                                </li>
                            </For>
                        </ul>
                    </article>
                </li>
            </For>
        </ul>
    }
}
