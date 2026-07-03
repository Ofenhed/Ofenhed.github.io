pub mod ai;
pub mod metadata;
pub mod path;
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
        unremarkable::Unremarkable,
    },
    helpers::{AddContext, ForRoute, ZWNJ, context_signal, into_static_str},
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
    let mut ret = vec![PathSegment::Static(Cow::Borrowed("clog"))];
    owner.child().with(|| {
        f();
        if let Some(tag) = use_context::<Signal<Option<TagFilter>>>()
            .map(|x| x.get())
            .flatten()
        {
            tag.generate_path(&mut ret);
        }
        use_context::<Signal<SortBy>>()
            .unwrap()
            .get()
            .generate_path(&mut ret);
        use_context::<Signal<SortInvert>>()
            .unwrap()
            .get()
            .generate_path(&mut ret);
        use_context::<Signal<CurrentPage>>()
            .unwrap()
            .get()
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
            [b.with_blog(Unremarkable)]
        }
    };
    published.into_iter().chain(unpublished.into_iter())
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
    let current_page =
        use_context::<Signal<CurrentPage>>().expect("Current page always defined here");
    let blogs = use_context::<Signal<FilteredEntities>>()
        .expect("Filtered entities are always defined here");
    let num_pages = |entries| {
        let entry_count = entries;
        let pages_full = entry_count + ENTRIES_PER_PAGE - 1;
        let pages = pages_full / ENTRIES_PER_PAGE;
        pages
    };
    let num_pages = move || num_pages(blogs.with(|x| x.0.len()));
    let previous_page = move || {
        current_page.with(move |CurrentPage(p)| {
            let p = *p;
            if p != 0 {
                Some(view! {
                    <a href=current_url_with(move || provide_context(
                        Signal::derive(move || CurrentPage(p - 1)),
                    ))>"<"</a>
                })
            } else {
                None
            }
        })
    };
    let next_page = move || {
        current_page.with(move |CurrentPage(p)| {
            let p = *p + 1;
            let num_pages = num_pages();
            if p < num_pages {
                Some(view! {
                    <a href=current_url_with(move || provide_context(
                        Signal::derive(move || CurrentPage(p)),
                    ))>">"</a>
                })
            } else {
                None
            }
        })
    };
    let pagination = move || {
        let num_pages = num_pages();
        let is_current = move |p| {
            move || {
                if current_page.with(|CurrentPage(c)| p == *c) {
                    Some("page")
                } else {
                    None
                }
            }
        };
        if num_pages > 1 {
            Some(view! {
                <div class="pagination">
                    {previous_page} <For each=move || 0..num_pages key=|x| *x let(page)>
                        <a
                            aria-current=is_current(page)
                            href=current_url_with(|| provide_context(
                                Signal::derive(move || CurrentPage(page)),
                            ))
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
            let CurrentPage(page) = current_page.get();
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
        let pages = pages_full / ENTRIES_PER_PAGE;
        pages
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
                            view! {
                                <AddContext context=CurrentPage(key) />
                                <Outlet />
                            }
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
                        view=move || view! { <AddContext context=key.to_owned() /> }
                        ssr=SsrMode::OutOfOrder
                    >
                        <ParentRoute
                            path=SortInvert(true)
                            view=|| view! { <AddContext context=SortInvert(true) /> }
                            ssr=SsrMode::OutOfOrder
                        >
                            <BlogPaging />
                        </ParentRoute>
                        <ParentRoute
                            path=SortInvert(false)
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
    }
    .into_inner()
}

#[component(transparent)]
pub fn BlogTagFilter() -> impl MatchNestedRoutes + Clone + 'static {
    let no_tag_filter: Option<TagFilter> = None;
    view! {
        <ParentRoute path=path!("") view=Outlet ssr=SsrMode::OutOfOrder>
            <ForRoute
                each=Tag::VARIANTS.iter().map(|x| TagFilter(*x))
                children=|key| {
                    view! {
                        <ParentRoute
                            path=key
                            view=move || {
                                view! { <AddContext context=Some(key) /> }
                            }
                            ssr=SsrMode::OutOfOrder
                        >
                            <BlogSorting />
                        </ParentRoute>
                    }
                        .into_inner()
                }
            />
            <ParentRoute
                path=path!("")
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
        let (sort_by, _) = context_signal(SortBy::Default);
        let (inverted, _) = context_signal(SortInvert(false));
        let (page, _) = context_signal(CurrentPage(0));
        let (tags, _) = context_signal(None::<TagFilter>);
        let filtered_entities = Signal::derive(move || {
            FilteredEntities(blogs.with(|b| {
                let filter = tags.get();
                b.into_iter()
                    .filter(|x| {
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
            path=path!("")
            view=move || {
                Effect::new(move |_| {
                    let blogs = blogs.with(|x| x.iter().map(|x| x.uid).collect());
                    for b in with_blogs(PreloadUids(blogs)).filter_map(|x| x) {
                        spawn_local_scoped(async move { b.await });
                    }
                });
                view! {
                    <BlogEntryList entries=blogs />
                    <Outlet />
                    <BlogPagingLinks />
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
            <time class="publish" datetime=B::PUBLISH_DATE.date_naive().to_string()>
                {B::PUBLISH_DATE.date_naive().to_string()}
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
        <section class="article-info">{publish}<span />{last_update}</section>
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
        toggle.get().map(|x| x.set_checked(false));
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
                                let mut path = vec![
                                    PathSegment::Static(Cow::Borrowed("clog")),
                                    PathSegment::Static(Cow::Borrowed("entry")),
                                ];
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
                                    <A href=move || {
                                        let mut path = vec![
                                            PathSegment::Static(Cow::Borrowed("clog")),
                                        ];
                                        TagFilter(*tag).generate_path(&mut path);
                                        format_path(path).into_owned()
                                    }>{into_static_str(tag)}</A>
                                </li>
                            </For>
                        </ul>
                    </article>
                </li>
            </For>
        </ul>
    }
}
