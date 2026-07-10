pub mod ai;
pub mod chat_control;
pub mod metadata;
pub mod path;
#[cfg(debug_assertions)]
pub mod unremarkable;
pub mod why;

use crate::{
    app::HamburgerMenu,
    blog::{
        metadata::{
            BlogEntry, BlogEntryHandler, BlogEntryHandlerFor, Locale, PreloadUids, Tag,
            with_blog_simple,
        },
        path::format_path,
    },
    helpers::{ForRoute, ZWNJ, into_static_str, reset_footnote},
};
use chrono::{DateTime, Utc};
use leptos::{
    attr::custom::custom_attribute, either::Either, ev, prelude::*, task::spawn_local_scoped,
};
use leptos_meta::{Meta, use_head};
#[allow(unused)] // False positive
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    Lazy, PathSegment, PossibleRouteMatch, SsrMode,
    any_nested_route::{AnyNestedRoute, IntoAnyNestedRoute},
    components::{A, ParentRoute, Route},
    nested_router::Outlet,
    path,
    static_routes::StaticRoute,
};
use std::{borrow::Cow, str::FromStr};
use strum::{EnumString, IntoStaticStr, VariantArray};

const ENTRIES_PER_PAGE: usize = 10;

fn current_path_with(f: impl Fn()) -> Vec<PathSegment> {
    let owner = Owner::current().unwrap();
    let mut ret = vec![PathSegment::Static(Cow::Borrowed("clogs"))];
    owner.child().with(|| {
        f();
        use_context::<SortBy>().unwrap().generate_path(&mut ret);
        use_context::<SortInvert>().unwrap().generate_path(&mut ret);
        if let Some(tag) = use_context::<Option<TagFilter>>().unwrap() {
            tag.generate_path(&mut ret);
        }
        use_context::<CurrentPage>()
            .unwrap()
            .generate_path(&mut ret);
    });
    ret
}

fn current_url_with(f: impl Fn()) -> String {
    format_path(current_path_with(f)).into_owned()
}

pub fn with_blogs<B: BlogEntryHandler>(mut b: B) -> impl Iterator<Item = B::Result> {
    let published = [b.with_blog(why::WhyBlog), b.with_blog(ai::WhatAreLLMs)];
    let unpublished = {
        #[cfg(not(debug_assertions))]
        {
            []
        }
        #[cfg(debug_assertions)]
        {
            [
                b.with_blog(unremarkable::Unremarkable),
                b.with_blog(chat_control::ChatControl),
            ]
        }
    };
    published.into_iter().chain(unpublished)
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
        view! {
            <ParentRoute
                path=metadata
                view=move || {
                    reset_footnote();
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
    }
}

#[component]
pub(crate) fn BlogPagingLinks() -> impl IntoView {
    let num_pages = |entries| {
        let entry_count = entries;
        let pages_full = entry_count + ENTRIES_PER_PAGE - 1;
        pages_full / ENTRIES_PER_PAGE
    };
    let num_pages = move || {
        let FilteredEntities(blogs) =
            use_context().expect("Filtered entities are always defined here");
        num_pages(blogs.len())
    };
    let previous_page = move || {
        let CurrentPage(p) = use_context().expect("Current page always defined here");
        if p != 0 {
            Some(
                view! { <a href=current_url_with(move || provide_context(CurrentPage(p - 1)))>"<"</a> },
            )
        } else {
            None
        }
    };
    let next_page = move || {
        let CurrentPage(p) = use_context().expect("Current page always defined here");
        let num_pages = num_pages();
        if p < num_pages {
            Some(
                view! { <a href=current_url_with(move || provide_context(CurrentPage(p + 1)))>">"</a> },
            )
        } else {
            None
        }
    };
    let pagination = move || {
        let num_pages = num_pages();
        let is_current = move |p| {
            move || {
                let CurrentPage(c) = use_context().expect("Current page always defined here");
                if p == c { Some("page") } else { None }
            }
        };
        if num_pages > 1 {
            Some(view! {
                <div class="pagination">
                    {previous_page} <For each=move || 0..num_pages key=|x| *x let(page)>
                        <a
                            aria-current=is_current(page)
                            href=current_url_with(|| provide_context(CurrentPage(page)))
                        >
                            {page + 1}
                        </a>
                    </For> {next_page}
                </div>
            })
        } else {
            None
        }
    };
    #[cfg(not(feature = "statics"))]
    let maybe_ignore = ();
    #[cfg(feature = "statics")]
    let maybe_ignore = {
        let leptos_static_files::NoGenerateStatic(ignore) =
            use_context().expect("This should be defined by static_files");
        Signal::derive(move || {
            let CurrentPage(page) = use_context().expect("Current page always defined here");
            let page = page + 1;
            let last_page = num_pages();
            if last_page < page {
                ignore.set(true);
            }
        })
    };
    (maybe_ignore, pagination)
}

#[component(transparent)]
pub fn BlogPaging() -> impl MatchNestedRoutes + Clone + 'static {
    let num_pages = |entries| {
        let entry_count = entries;
        let pages_full = entry_count + ENTRIES_PER_PAGE - 1;
        pages_full / ENTRIES_PER_PAGE
    };
    let max_pages = num_pages(with_blogs(()).count());
    view! {
        <ForRoute
            each=0..max_pages
            children=move |key| {
                view! {
                    <Route
                        path=CurrentPage(key)
                        view=move || {
                            provide_context(CurrentPage(key));
                            view! { <BlogListing /> }.into_inner()
                        }
                        ssr=SsrMode::Static(StaticRoute::new())
                    />
                }
                    .into_inner()
            }
        />
    }
    .into_inner()
}

#[component(transparent)]
pub fn BlogSorting() -> impl MatchNestedRoutes + Clone + 'static {
    view! {
        <ForRoute
            each=SortBy::VARIANTS.iter()
            children=|key| {
                view! {
                    <ParentRoute
                        path=*key
                        view=move || {
                            provide_context(key.to_owned());
                            view! { <Outlet /> }
                        }
                        ssr=SsrMode::OutOfOrder
                    >
                        <ParentRoute
                            path=SortInvert(true)
                            view=|| {
                                provide_context(SortInvert(true));
                                view! { <Outlet /> }
                            }
                            ssr=SsrMode::OutOfOrder
                        >
                            <BlogTagFilter />
                        </ParentRoute>
                        <ParentRoute
                            path=SortInvert(false)
                            view=|| {
                                provide_context(SortInvert(false));
                                view! { <Outlet /> }
                            }
                            ssr=SsrMode::OutOfOrder
                        >
                            <BlogTagFilter />
                        </ParentRoute>
                    </ParentRoute>
                }
                    .into_inner()
            }
        />
    }
    .into_inner()
}

#[component(transparent)]
pub fn BlogTagFilter() -> impl MatchNestedRoutes + Clone + 'static {
    view! {
        <ParentRoute path=path!("") view=Outlet ssr=SsrMode::OutOfOrder>
            <ForRoute
                each=Tag::VARIANTS.iter().map(|x| TagFilter(*x))
                children=|key| {
                    view! {
                        <ParentRoute
                            path=key
                            view=move || {
                                provide_context(Some(key));
                                view! { <Outlet /> }
                            }
                            ssr=SsrMode::OutOfOrder
                        >
                            <BlogPaging />
                        </ParentRoute>
                    }
                        .into_inner()
                }
            />
            <ParentRoute
                path=path!("")
                view=move || {
                    provide_context(None::<TagFilter>);
                    view! { <Outlet /> }
                }
                ssr=SsrMode::OutOfOrder
            >
                <BlogPaging />
            </ParentRoute>
        </ParentRoute>
    }
    .into_inner()
}

#[component]
pub fn BlogListing() -> impl IntoView {
    let blogs = {
        let blogs = with_blogs_simple::<BlogEntryMeta>().collect::<Vec<_>>();
        let sort_by = use_context::<SortBy>().unwrap();
        let SortInvert(invert_sort) = use_context().unwrap();
        let CurrentPage(current_page) = use_context().unwrap();
        let tags = use_context::<Option<TagFilter>>().unwrap();
        let filtered_entities = FilteredEntities(
            blogs
                .iter()
                .filter(|x| {
                    if let Some(TagFilter(filter)) = tags {
                        x.tags.contains(&filter)
                    } else {
                        true
                    }
                })
                .cloned()
                .collect::<Vec<_>>()
                .into(),
        );
        provide_context(filtered_entities.clone());
        let entries = Signal::derive(move || {
            let FilteredEntities(blogs) = use_context().unwrap();
            let mut blogs = blogs.into_owned();
            blogs.sort_unstable_by(|a, b| {
                let (a, b) = if invert_sort { (b, a) } else { (a, b) };
                match sort_by {
                    SortBy::Default => match (&a.pin, &b.pin) {
                        (x, y) if x == y => b.publish_date.partial_cmp(&a.publish_date).unwrap(),
                        (None, None) => unreachable!("None == None"),
                        (Some(_), None) => std::cmp::Ordering::Less,
                        (None, Some(_)) => std::cmp::Ordering::Greater,
                        (Some(x), Some(y)) => x.cmp(y),
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
            let (chunks, tail) = blogs.as_chunks::<ENTRIES_PER_PAGE>();
            match chunks.len() {
                x if current_page == x => tail,
                x if current_page > x => &blogs[..],
                _ => &chunks[current_page],
            }
            .to_vec()
        });
        provide_context(CurrentPageEntries(entries.get()));
        entries
    };
    Effect::new(|| {
        let FilteredEntities(blogs) = use_context().unwrap();
        for b in with_blogs(PreloadUids(blogs.iter().map(|x| x.uid).collect())).flatten() {
            spawn_local_scoped(b);
        }
    });

    view! {
        <BlogEntryList entries=blogs />
        <BlogPagingLinks />
    }
}

#[component(transparent)]
pub fn Blog() -> impl MatchNestedRoutes + Clone {
    let blogs = with_blogs_simple::<AnyNestedRoute>().collect::<Vec<_>>();
    view! {
        <ParentRoute path=path!("") view=Outlet ssr=SsrMode::OutOfOrder>
            <ParentRoute path=path!("/clog") view=Outlet ssr=SsrMode::OutOfOrder>
                <ForRoute each=blogs children=|b| b />
            </ParentRoute>
            <ParentRoute path=path!("/clogs") view=Outlet ssr=SsrMode::OutOfOrder>
                <BlogSorting />
            </ParentRoute>
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
    path_locale: bool,
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
            uid: T::UID,
            publish_date: T::PUBLISH_DATE,
            last_updated: T::LAST_UPDATED,
            locale: T::LOCALE,
            path_locale: T::PATH_LOCALE,
            title: T::TITLE,
            tags: T::TAGS,
            pin: T::PIN,
        }
    }
}

#[component]
pub(crate) fn BlogHeading<B: BlogEntry>(entry: B) -> impl IntoView {
    use leptos_meta::Title;
    _ = entry;
    use_head();
    let last_update = B::LAST_UPDATED.map(|x| {
        let mut date: Oco<'static, str> = Oco::Owned(x.date_naive().to_string());
        date.upgrade_inplace();
        view! {
            <Meta property="og:modified_time" content=x.to_rfc3339() />
            <time class="update" datetime=date.clone()>
                {date.clone()}
            </time>
        }
    });
    let publish = {
        let mut date: Oco<'static, str> = Oco::Owned(B::PUBLISH_DATE.date_naive().to_string());
        date.upgrade_inplace();
        view! {
            <Meta property="og:article:published_time" content=B::PUBLISH_DATE.to_rfc3339() />
            <time class="publish" datetime=date.clone()>
                {date.clone()}
                {ZWNJ}
            </time>
        }
    };
    let locale = B::LOCALE.map(|x| {
        view! { <Meta property="og:locale" content=into_static_str(x) /> }
    });
    view! {
        <Title formatter=|title: String| format!("{title} - Captains Log") text=B::TITLE />
        {locale}
        <Meta property="og:title" content=B::TITLE />
        <Meta property="og:article:author" content="Marcus Ofenhed" />
        <For
            each=move || B::TAGS.iter()
            key=|x| x.to_owned()
            children=|tag| {
                view! { <Meta property="og:article:tag" content=into_static_str(tag) /> }
            }
        />
        <h1 id="pageHeader">{B::TITLE}</h1>
        <section class="article-info">{publish} {last_update}</section>
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
struct FilteredEntities(Oco<'static, [BlogEntryMeta]>);

#[allow(unused)]
struct CurrentPageEntries(Vec<BlogEntryMeta>);

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy, Default, PartialEq, Eq, Hash, IntoStaticStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub(crate) enum SortBy {
    #[default]
    Default,
    PublishDate,
    ModifyDate,
    Title,
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Default, PartialEq, Eq)]
pub(crate) struct SortInvert(bool);

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone, Copy)]
pub(crate) struct TagFilter(Tag);

#[derive(Clone, Copy)]
pub(crate) struct CurrentPage(usize);

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
        if let Some(x) = toggle.get() {
            x.set_checked(false);
        }
    };
    let article_pinned = |entry: &BlogEntryMeta| {
        entry
            .pin
            .map(|pin| Either::Right(custom_attribute("pinned", pin)))
            .unwrap_or(Either::Left(()))
    };
    let lang = move |meta: &BlogEntryMeta| {
        meta.locale
            .map(|x| {
                let s = into_static_str(x);
                let lang = match s.replace("_", "-") {
                    new if new == s => Cow::Borrowed(s),
                    new => Cow::Owned(new),
                };
                Either::Right(view! { <{..} lang=lang /> })
            })
            .unwrap_or(Either::Left(()))
    };
    let info = |meta: &BlogEntryMeta| {
        let update = meta.last_updated.map(|time| {
            let mut date: Oco<'static, str> = Oco::Owned(time.date_naive().to_string());
            date.upgrade_inplace();
            view! {
                <time class="update" datetime=date.clone()>
                    {date.clone()}
                </time>
            }
        });
        let mut date: Oco<'static, str> = Oco::Owned(meta.publish_date.date_naive().to_string());
        date.upgrade_inplace();
        view! {
            <span class="article-info">
                <time class="publish" datetime=date.clone()>
                    {date.clone()}
                    {ZWNJ}
                </time>
                {update}
            </span>
        }
    };
    view! {
        <ul id="blog-entries">
            <For each=move || entries.get() key=|x: &BlogEntryMeta| x.uid let(entry)>
                <li {..article_pinned(&entry)}>
                    <article {..lang(&entry)}>
                        <A
                            on:click=on_click
                            href={
                                let mut path = vec![PathSegment::Static(Cow::Borrowed("clog"))];
                                entry.generate_path(&mut path);
                                format!("{}#{}", format_path(path), to_title(entry.title))
                            }
                        >
                            {entry.title.to_owned()}
                        </A>
                        {info(&entry)}
                        <ul class="tags">
                            <For each=move || entry.tags key=|x| x.to_owned() let(tag)>
                                <li>
                                    <A href=current_url_with(|| {
                                        provide_context(Some(TagFilter(*tag)));
                                        provide_context(CurrentPage(0));
                                    })>{into_static_str(tag)}</A>
                                </li>
                            </For>
                        </ul>
                    </article>
                </li>
            </For>
        </ul>
    }
}
