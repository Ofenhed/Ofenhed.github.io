pub mod unremarkable;

use crate::blog::unremarkable::Unremarkable;
use chrono::{DateTime, Utc};
use leptos::{prelude::*, tachys::view::iterators::StaticVec};
use leptos_meta::{Meta, use_head};
use leptos_router::{
    MatchNestedRoutes, PartialPathMatch, PathSegment, PossibleRouteMatch, SsrMode,
    any_nested_route::IntoAnyNestedRoute,
    components::{A, ParentRoute, Route},
    nested_router::Outlet,
    params::Params,
    path,
    static_routes::StaticRoute,
};
use std::{borrow::Cow, str::FromStr};

#[derive(Params, Clone, Debug, PartialEq, Eq, Copy)]
pub struct BlogParams {
    id: u32,
}

#[derive(Clone)]
pub struct BlogContext {
    title: WriteSignal<&'static str>,
}

#[component]
fn BlogIndex() -> impl IntoView {
    if let Some(c) = use_context::<BlogContext>() {
        c.title.update(|t| *t = "Blog");
    }
    view! {
        <h1>"Posts"</h1>
    }
}

#[component(transparent)]
pub fn BlogRoute(
    blog: impl 'static + Send + Clone + Fn() -> PopulatedBlogEntry,
) -> impl leptos_router::MatchNestedRoutes + Clone {
    let metadata = blog().metadata();
    view! {
        <Route
            path=metadata
            view=move|| view!{
                <ShowBlogEntry entry=blog() />
            }
            ssr=SsrMode::Static(StaticRoute::new())
            />
    }
    .into_inner()
    .into_any_nested_route()
}

#[component(transparent)]
pub fn ForRoute<X, R: Send + 'static + MatchNestedRoutes + Clone, F: Fn(X) -> R>(
    each: impl IntoIterator<Item = X>,
    child: F,
) -> leptos_router::any_nested_route::AnyNestedRoute {
    let entries = StaticVec::from(each.into_iter().map(child).collect::<Vec<_>>());
    entries.into_any_nested_route()
}

#[component(transparent)]
pub fn Blog() -> impl MatchNestedRoutes + Clone {
    let blogs = vec![Unremarkable];
    let blog_metadata = blogs
        .iter()
        .map(|x| x().metadata())
        .collect::<Vec<EmptyBlogEntry>>();
    view! {
        <ParentRoute
            path=path!("/clog")
            view=Outlet
            ssr=SsrMode::Static(StaticRoute::new())
            clone:blog_metadata
            >
        <Route
            path=path!("/")
            view=move || view!{
                <BlogEntryList entries=blog_metadata.clone() />
            }.into_view()
            ssr=SsrMode::Static(StaticRoute::new()) />
            <ForRoute each=blogs child=move |b| view! {
                <BlogRoute blog=b />
            }.into_inner() />
        </ParentRoute>
    }
    .into_inner()
}

#[slot]
pub struct BlogEntry<T> {
    uid: u32,
    publish_date: DateTime<Utc>,
    last_updated: Option<DateTime<Utc>>,
    title: &'static str,
    tags: &'static [&'static str],
    children: T,
}

impl<T: Clone> Clone for BlogEntry<T> {
    fn clone(&self) -> Self {
        Self {
            uid: self.uid,
            publish_date: self.publish_date,
            last_updated: self.last_updated,
            title: self.title,
            tags: self.tags,
            children: self.children.clone(),
        }
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
        BlogEntry {
            uid: self.uid,
            publish_date: self.publish_date,
            last_updated: self.last_updated,
            title: self.title,
            tags: self.tags,
            children: (),
        }
    }
}

pub type PopulatedBlogEntry = BlogEntry<Children>;
pub type EmptyBlogEntry = BlogEntry<()>;

#[component]
pub fn ShowBlogEntry(entry: PopulatedBlogEntry) -> impl IntoView {
    use_head();
    let last_update = entry.last_updated.map(|x| {
        view! {
            <Meta property="og:modified_time" content=x.to_rfc3339() />
        }
    });
    view! {
        <For
            each=move || entry.tags.iter()
            key=|x| x.to_owned()
            children=|tag| view! {
        <Meta property="og:article:tag" content=tag.to_owned() />
            } />
        <Meta property="og:article:published_time" content=entry.publish_date.to_rfc3339() />
        {last_update}
        <h1>
        { entry.title }
        </h1>
        <p>{ entry.publish_date.to_string() }</p>
        { (entry.children)() }
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

#[component]
pub fn BlogEntryList(entries: Vec<EmptyBlogEntry>) -> impl IntoView {
    view! {
        <ul>
        <For each=move || entries.clone()
            key=|x: &EmptyBlogEntry| x.uid
            let(entry)>
            <li>
                <A href=move || format!("/clog/{}#{}", entry.uid, to_title(entry.title))>
                    { entry.title.to_string() }
                </A>
            </li>
        </For>
        </ul>
    }
}
