use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::StreamExt as _;
use leptos::{config::LeptosOptions, prelude::*};
use reactive_graph::owner::Sandboxed;

pub mod prelude {
    pub use super::{StaticFileGeneratorError, StaticFileOptions, NoGenerateStatic};
}

pub struct StaticFileOptions<'a, C> {
    pub leptos: Oco<'a, LeptosOptions>,
    pub additional_context: C,
    pub excluded_routes: Vec<String>,
}

#[derive(Clone)]
pub struct NoGenerateStatic(pub WriteSignal<bool>);

impl<'a> StaticFileOptions<'a, ()> {
    pub fn new(
        leptos: impl Into<Oco<'a, LeptosOptions>>,
    ) -> StaticFileOptions<'a, impl Fn() + Clone + Send + 'static> {
        StaticFileOptions {
            leptos: leptos.into(),
            additional_context: || (),
            excluded_routes: vec![],
        }
    }

    pub fn with_additional_context<C: Fn() + Clone + Send + 'a>(
        self,
        f: C,
    ) -> StaticFileOptions<'a, C> {
        StaticFileOptions {
            additional_context: f,
            leptos: self.leptos,
            excluded_routes: self.excluded_routes,
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StaticFileGeneratorError {
    #[error("Executor error: {0}")]
    Executor(#[from] any_spawner::ExecutorError),
    #[error("Invalid path: {0}")]
    InvalidPath(Oco<'static, str>),
    #[error("{0}")]
    Io(#[from] std::io::Error),
    #[error("Could not generate routes")]
    NoRoutesGenerated,
    #[error("Duplicate paths: {}", .0.join(" "))]
    DuplicatePath(Vec<String>),
}

fn init_executor() -> Result<(), any_spawner::ExecutorError> {
    #[cfg(feature = "tokio")]
    {
        any_spawner::Executor::init_tokio()
    }
    #[cfg(not(feature = "tokio"))]
    {
        compiler_error!("No supported executor chosen");
    }
}

fn static_path(
    leptos: &LeptosOptions,
    path: Oco<'_, str>,
) -> Result<Oco<'static, Path>, StaticFileGeneratorError> {
    let mut p = format!("./{}", path);
    if p.ends_with("/") {
        p.push_str("index");
    }
    p.push_str(".html");
    let p = PathBuf::from(p);
    if p.components()
        .find(|x| {
            use std::path::Component as C;
            match x {
                C::Normal(_) | C::CurDir => false,
                _ => true,
            }
        })
        .is_some()
    {
        return Err(StaticFileGeneratorError::InvalidPath(Oco::Owned(
            p.display().to_string(),
        )));
    }
    let root = Path::new(&*leptos.site_root);
    let mut result = root.to_path_buf();
    result.push(p);
    if !result.starts_with(root) {
        return Err(StaticFileGeneratorError::InvalidPath(Oco::Owned(
            result.display().to_string(),
        )));
    }
    Ok(Oco::Owned(result))
}

struct IterDup<I, E> {
    last: Option<E>,
    iter: I,
}

impl<E: Eq, I: Iterator<Item = E>> From<I> for IterDup<I, E> {
    fn from(iter: I) -> Self {
        Self {
            last: None,
            iter: iter,
        }
    }
}

impl<E: Eq, I: Iterator<Item = E>> Iterator for IterDup<I, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        match (self.iter.next(), &self.last) {
            (Some(x), Some(y)) if x == *y => Some(x),
            (None, _) => None,
            (Some(x), _) => {
                self.last = Some(x);
                self.next()
            }
        }
    }
}

impl<'a, C: Fn() + Clone + Send + 'static> StaticFileOptions<'a, C> {
    pub async fn generate_static_files<IV: IntoView>(
        &self,
        app_fn: impl Clone + Send + 'static + Fn() -> IV,
    ) -> Result<(), StaticFileGeneratorError> {
        init_executor()?;
        let new_owner =
            || Owner::new_root(Some(Arc::from(hydration_context::SsrSharedContext::new())));
        let routes = {
            let owner = new_owner();
            let list = owner.with({
                let additional_context = self.additional_context.clone();
                let not_found_path = self.leptos.not_found_path.clone();
                let app_fn = app_fn.clone();
                move || {
                    provide_context(leptos_router::location::RequestUrl::new(""));
                    #[cfg(feature = "meta")]
                    {
                        use leptos_meta::*;
                        let (mock_meta, _) = ServerMetaContext::new();
                        provide_context(mock_meta);
                        let (_, mock_writer) = signal(false);
                        provide_context(NoGenerateStatic(mock_writer));
                    }
                    additional_context();
                    leptos_router::RouteList::generate(app_fn).map(|mut list| {
                        use leptos_router::{
                            Method, PathSegment, RouteListing, SsrMode, static_routes::StaticRoute,
                        };
                        let route_404 = RouteListing::new(
                            [PathSegment::Static(Cow::Owned(not_found_path.to_string()))],
                            SsrMode::Static(StaticRoute::new()),
                            [Method::Get],
                            [],
                        );
                        if list.iter().find(|x| x.path() == route_404.path()).is_none() {
                            list.push(route_404);
                        }
                        list
                    })
                }
            });
            owner.unset_with_forced_cleanup();
            list
        };

        let known_paths = Box::leak(
            {
                #[cfg(feature = "tokio")]
                {
                    tokio::sync::Mutex::new(Vec::new())
                }
                #[cfg(not(feature = "tokio"))]
                {
                    std::sync::Mutex::new(Vec::new())
                }
            }
            .into(),
        );
        let known_paths = || {
            #[cfg(feature = "tokio")]
            {
                known_paths.lock()
            }
            #[cfg(not(feature = "tokio"))]
            {
                let l = known_paths.lock();
                async move { l }
            }
        };

        for route in routes
            .ok_or(StaticFileGeneratorError::NoRoutesGenerated)?
            .into_inner()
        {
            #[derive(Clone)]
            struct NoGenerateStaticReader(ReadSignal<bool>);
            route
                .clone()
                .generate_static_files(
                    {
                        let additional_context = self.additional_context.clone();
                        let app_fn = app_fn.clone();
                        move |path| {
                            let path = path.clone();
                            let additional_context = additional_context.clone();
                            let app_fn = app_fn.clone();
                            Sandboxed::new(async move {
                                eprintln!("Building {path}");
                                known_paths().await.push(path.to_string());
                                let owner = new_owner();
                                let inject_meta = owner.with(move || {
                                    provide_context(leptos_router::location::RequestUrl::new(
                                        path.as_ref(),
                                    ));
                                    let (read_ignored, write_ignored) = signal(false);
                                    provide_context(NoGenerateStatic(write_ignored));
                                    provide_context(NoGenerateStaticReader(read_ignored));
                                    #[cfg(feature = "meta")]
                                    {
                                        use leptos_meta::*;
                                        let (mock_meta, set_meta) = ServerMetaContext::new();
                                        provide_context(mock_meta);
                                        additional_context();
                                        |x| set_meta.inject_meta_context(x)
                                    }
                                    #[cfg(not(feature = "meta"))]
                                    |x| async move { x }
                                });
                                let reply = owner
                                    .with(move || {
                                        let v = (app_fn)();
                                        async move {
                                            let v = v.resolve().await;
                                            let stream = inject_meta(
                                                v.resolve().await.to_html_stream_in_order(),
                                            )
                                            .await;
                                            stream.collect().await
                                        }
                                    })
                                    .await;

                                (owner, reply)
                            })
                        }
                    },
                    {
                        let leptos_options = self.leptos.clone();
                        move |path, owner, content| {
                            let leptos_options = leptos_options.clone();
                            let target = static_path(&leptos_options, Oco::Borrowed(path.as_ref()))
                                .map_err(std::io::Error::other);
                            let path = path.to_owned();
                            let child = owner.child();
                            async move {
                                let dont_generate = child.with(|| {
                                    let NoGenerateStaticReader(r) =
                                        use_context().expect("Inserted above");
                                    r.get()
                                });
                                if dont_generate {
                                    eprintln!("Ignoring path {path}");
                                    return Ok(());
                                }
                                let target = target?;
                                eprintln!("Saving {path}");
                                #[cfg(feature = "tokio")]
                                {
                                    if let Some(parent) = target.parent() {
                                        tokio::fs::create_dir_all(parent).await?;
                                    }
                                    tokio::fs::write(target, content).await?;
                                    Ok(())
                                }
                                #[cfg(not(feature = "tokio"))]
                                {
                                    unimplemented!()
                                }
                            }
                        }
                    },
                    |_owner| false,
                )
                .await;
        }
        let mut known_paths = known_paths().await;
        known_paths.sort_unstable();
        let mut duplicates = IterDup::from(known_paths.iter()).peekable();
        if let Some(_) = duplicates.peek() {
            Err(StaticFileGeneratorError::DuplicatePath(
                duplicates.cloned().collect(),
            ))
        } else {
            Ok(())
        }
    }
}
