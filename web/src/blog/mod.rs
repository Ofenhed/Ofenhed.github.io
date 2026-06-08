pub mod metadata;
pub mod unremarkable;

use crate::{
    blog::{
        metadata::{Locale, Tag},
        unremarkable::Unremarkable,
    },
    helpers::{AddContext, ForRoute},
};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use leptos_meta::{Meta, use_head};
#[allow(unused)] // False positive
use leptos_router::MatchNestedRoutes;
use leptos_router::{
    PartialPathMatch, PathSegment, PossibleRouteMatch, SsrMode, StaticSegment,
    any_nested_route::IntoAnyNestedRoute,
    components::{A, ParentRoute, Route},
    hooks::use_params,
    nested_router::Outlet,
    params::Params,
    path,
    static_routes::StaticRoute,
};
use std::{
    borrow::Cow,
    cmp::max,
    str::FromStr,
    sync::{OnceLock, RwLock},
};
use strum::{AsRefStr, EnumString, VariantArray};

const ENTRIES_PER_PAGE: usize = 10;
const BLOGS: &[fn() -> BlogEntry<Children>] = &[Unremarkable];

#[component(transparent)]
pub fn BlogRoute(
    blog: impl 'static + Send + Clone + Fn() -> PopulatedBlogEntry,
) -> impl MatchNestedRoutes + Clone {
    let metadata = blog().metadata();
    view! {
        <Route
            path=metadata
            view=move || view! { <ShowBlogEntry entry=blog() /> }
            ssr=SsrMode::Static(StaticRoute::new())
        />
    }
    .into_inner()
    .into_any_nested_route()
}

pub fn num_strings() -> &'static RwLock<Vec<(usize, &'static str)>> {
    static NUM_CELL: OnceLock<RwLock<Vec<(usize, &'static str)>>> = OnceLock::new();
    NUM_CELL.get_or_init(Default::default)
}

pub fn to_static_str(from: usize) -> &'static str {
    let lock = num_strings().read().unwrap();
    if let Ok(index) = lock.binary_search_by_key(&from, |(i, _)| *i) {
        lock[index].1
    } else {
        drop(lock);
        let mut lock = num_strings().write().unwrap();
        if let Ok(index) = lock.binary_search_by_key(&from, |(i, _)| *i) {
            lock[index].1
        } else {
            let num = Box::leak(Box::from(format!("{from}")));
            lock.push((from, num));
            lock.sort_unstable_by_key(|(i, _)| *i);
            lock.dedup_by_key(|(i, _)| *i);
            num
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
            <ParentRoute
                path=path!("")
                view=Outlet
                ssr=SsrMode::OutOfOrder>
                <Route
                  path=path!("/page/:page")
                  view = move || {
                      let page = page().get();
                      view! {
                        <AddContext context=page />
            {maybe_ignore.clone()}
                      }
                  }
                  ssr=SsrMode::Static(StaticRoute::new().prerender_params(
                // TODO: There is a leptos bug that strips the static route prerender of
                // children if parent routes has a static ssr. This should probably be
                // moved to BlogPaging, at which point the count should be handled
                // dynamically based on filtered entries
                move || {
                    async move {
                        let max_pages = num_pages(BLOGS.len());
                        [(
                            "page".to_string(),
                            (1..max_pages).map(|x| (x + 1).to_string()).collect::<Vec<_>>(),
                        )]
                        .into_iter()
                        .collect()
                    }
                },
            ))
                          />
                <Route
                    path=path!("/")
                    view=move || view! { <AddContext context=CurrentPage(0) /> {maybe_ignore}
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
            <ParentRoute
                path=path!("")
                view=Outlet
                ssr=SsrMode::OutOfOrder>
                <ParentRoute
                    path=path!("/sort")
                    view=Outlet
                    ssr=SsrMode::OutOfOrder
                    >
                    <ForRoute each=SortBy::VARIANTS.iter() children=|key| view! {
                    <ParentRoute
                        path=(StaticSegment(key.as_ref()), )
                        view=move || view! { <AddContext context=key.to_owned() /> }
                        ssr=SsrMode::OutOfOrder
                        >
                        <ParentRoute
                            path=path!("/invert")
                            view=|| view! { <AddContext context=SortInvert(true) /> }
                            ssr=SsrMode::OutOfOrder>
                            <BlogPaging />
                        </ParentRoute>
                        <ParentRoute
                            path=path!("/")
                            view=|| view! { <AddContext context=SortInvert(false) /> }
                            ssr=SsrMode::OutOfOrder>
                            <BlogPaging />
                        </ParentRoute>
                    </ParentRoute>
                    }.into_inner() />
                </ParentRoute>
                <ParentRoute
                    path=path!("/")
                    view=|| view! { <AddContext context=SortInvert(false)>
                        <AddContext context=SortBy::PublishDate />
                            </AddContext>
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
    view! {
            <ParentRoute
                path=path!("")
                view=Outlet
                ssr=SsrMode::OutOfOrder>
                <ParentRoute
                    path=path!("/tag")
                    view=Outlet
                    ssr=SsrMode::OutOfOrder
                    >
                    <ForRoute each=Tag::VARIANTS.iter() children=|key| view! {
                        <ParentRoute
                            path=(StaticSegment(key.as_ref()), )
                            view=move || view! { <AddContext context=Some(TagFilter(key.to_owned())) /> }
                            ssr=SsrMode::OutOfOrder
                            >
                            <BlogSorting />
                        </ParentRoute>
                    }.into_inner() />
                </ParentRoute>
                <ParentRoute
                    path=path!("/")
                    view=|| view! {
                        <AddContext context={None::<TagFilter>} />
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
pub fn BlogListing(
    #[prop(into)] blogs: Signal<Vec<EmptyBlogEntry>>,
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
                        if let Some(TagFilter(filter)) = filter {
                            x.tags.contains(&filter)
                        } else {
                            true
                        }
                    })
                    .map(ToOwned::to_owned)
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
                view=move || view! { <Outlet /><BlogEntryList entries=blogs /> }
                  ssr=SsrMode::OutOfOrder
                >
                <BlogTagFilter />
            </ParentRoute>
    }
    .into_inner()
}
#[component(transparent)]
pub fn Blog() -> impl MatchNestedRoutes + Clone {
    let blogs = BLOGS.to_vec();
    let blog_metadata = BLOGS
        .iter()
        .map(|x| x().metadata())
        .collect::<Vec<EmptyBlogEntry>>();
    view! {
        <ParentRoute
            path=path!("/clog")
            view=Outlet
            ssr=SsrMode::OutOfOrder
        >
            <ParentRoute
                path=path!("/entry")
                view=Outlet
                ssr=SsrMode::Static(StaticRoute::new())>
                <ForRoute each=blogs children=|b| view! { <BlogRoute blog=b /> }.into_inner() />
            </ParentRoute>
            <BlogListing blogs=blog_metadata />
        </ParentRoute>
    }
    .into_inner()
}

#[slot]
#[derive(PartialEq, Eq)]
pub struct BlogEntry<T> {
    uid: u32,
    publish_date: DateTime<Utc>,
    last_updated: Option<DateTime<Utc>>,
    locale: Option<Locale>,
    title: &'static str,
    tags: &'static [Tag],
    children: T,
}

impl<T> BlogEntry<T> {
    fn clone_map<R>(&self, f: impl FnOnce(&T) -> R) -> BlogEntry<R> {
        let BlogEntry {
            uid,
            publish_date,
            last_updated,
            locale,
            title,
            tags,
            children,
        } = self;
        BlogEntry {
            uid: *uid,
            publish_date: publish_date.clone(),
            last_updated: last_updated.clone(),
            locale: locale.clone(),
            title: title,
            tags: tags,
            children: (),
        }
        .map(|_| f(children))
    }

    fn map<R>(self, f: impl FnOnce(T) -> R) -> BlogEntry<R> {
        let BlogEntry {
            uid,
            publish_date,
            last_updated,
            locale,
            title,
            tags,
            children,
        } = self;
        BlogEntry {
            uid,
            publish_date,
            last_updated,
            locale,
            title,
            tags,
            children: f(children),
        }
    }
}

impl<T: Clone> Clone for BlogEntry<T> {
    fn clone(&self) -> Self {
        self.clone_map(|x| x.clone())
    }
}

impl<T> PossibleRouteMatch for BlogEntry<T> {
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

impl<T> BlogEntry<T> {
    pub fn metadata(&self) -> BlogEntry<()> {
        self.clone_map(|_| ())
    }
}

pub type PopulatedBlogEntry = BlogEntry<Children>;
pub type EmptyBlogEntry = BlogEntry<()>;

#[component]
pub fn ShowBlogEntry(entry: PopulatedBlogEntry) -> impl IntoView {
    use leptos_meta::Title;
    use_head();
    let last_update = entry.last_updated.map(|x| {
        view! { <Meta property="og:modified_time" content=x.to_rfc3339() /> }
    });
    let locale = entry.locale.map(|x| {
        view! { <Meta property="og:locale" content=x.as_ref().to_string() /> }
    });
    view! {
        <Title formatter=|title: String| format!("{title} - Captains Log") text=entry.title />
        {locale}
        <Meta property="og:title" content=entry.title />
        <Meta property="og:article:author" content="Marcus Ofenhed" />
        <For
            each=move || entry.tags.iter()
            key=|x| x.to_owned()
            children=|tag| view! { <Meta property="og:article:tag" content=tag.as_ref().to_string() /> }
        />
        <Meta property="og:article:published_time" content=entry.publish_date.to_rfc3339() />
        {last_update}
        <h1>{entry.title}</h1>
        <p>{entry.publish_date.to_string()}</p>
        {(entry.children)()}
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
struct FilteredEntities(Vec<EmptyBlogEntry>);

#[allow(unused)]
struct CurrentPageEntries(Vec<EmptyBlogEntry>);

#[derive(Clone, Default, Debug, PartialEq, Eq, Hash, AsRefStr, VariantArray, EnumString)]
#[strum(serialize_all = "kebab-case")]
pub enum SortBy {
    #[default]
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
pub fn BlogEntryList(#[prop(into)] entries: Signal<Vec<EmptyBlogEntry>>) -> impl IntoView {
    view! {
        <ul id="blog-entries">
            <For each=move || entries.get() key=|x: &EmptyBlogEntry| x.uid let(entry)>
                <li>
                    <article>
                    <A href=move || {
                        format!("/clog/entry/{}#{}", entry.uid, to_title(entry.title))
                    }>{entry.title.to_owned()}</A>
                        <ul class="tags">
                            <For each=move || entry.tags key=|x| x.to_owned() let(tag)>
                                <li><A href=move || {
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
