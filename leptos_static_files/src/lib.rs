use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::Arc,
};

use futures::StreamExt as _;
use leptos::{config::LeptosOptions, prelude::*, server_fn::codec::IntoRes as _};
use reactive_graph::owner::Sandboxed;

pub mod prelude {
    pub use super::{StaticFileGeneratorError, StaticFileOptions};
}

pub struct StaticFileOptions<'a, C> {
    pub leptos: Oco<'a, LeptosOptions>,
    pub additional_context: C,
    pub excluded_routes: Vec<String>,
}

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
                x => {
                    println!("Found path component {x:?}");
                    true
                }
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

impl<'a, C: Fn() + Clone + Send + 'static> StaticFileOptions<'a, C> {
    pub async fn generate_static_files<IV: IntoView>(
        &self,
        app_fn: impl Clone + Send + 'static + Fn() -> IV,
    ) -> Result<(), StaticFileGeneratorError> {
        init_executor()?;
        let new_owner =
            || Owner::new_root(Some(Arc::from(hydration_context::SsrSharedContext::new())));
        let routes = new_owner().with({
            let additional_context = self.additional_context.clone();
            let not_found_path = self.leptos.not_found_path.clone();
            let app_fn = app_fn.clone();
            move || {
                provide_context(leptos_router::location::RequestUrl::new(""));
                //let (mock_parts, _) = Request::new(Body::from("")).into_parts();
                //provide_context(mock_parts);
                let (mock_meta, _) = leptos_meta::ServerMetaContext::new();
                provide_context(mock_meta);
                //provide_context(ResponseOptions::default());
                additional_context();
                leptos_router::RouteList::generate(app_fn.clone()).map(|mut list| {
                    use leptos_router::{
                        Method, PathSegment, RouteListing, SsrMode, static_routes::StaticRoute,
                    };
                    list.push(RouteListing::new(
                        [PathSegment::Static(Cow::Owned(not_found_path.to_string()))],
                        SsrMode::Static(StaticRoute::new()),
                        [Method::Get],
                        [],
                    ));
                    list
                })
            }
        });

        //let (something, iter) = routes.generate_routes();
        //for route in routes.iter() {
        //    println!("Got route {route:?}");
        //}
        routes
            .ok_or(StaticFileGeneratorError::NoRoutesGenerated)?
            .generate_static_files(
                {
                    let additional_context = self.additional_context.clone();
                    let app_fn = app_fn.clone();
                    move |path| {
                        let owner = new_owner();
                        let path = path.clone();
                        let additional_context = additional_context.clone();
                        let app_fn = app_fn.clone();
                        Sandboxed::new(async move {
                            let set_meta = owner.with(move || {
                                provide_context(leptos_router::location::RequestUrl::new(
                                    path.as_ref(),
                                ));
                                let (meta, set_meta) = leptos_meta::ServerMetaContext::new();
                                provide_context(meta);
                                additional_context();
                                set_meta
                            });
                            let sc = owner.shared_context().unwrap();
                            while let Some(pending) = sc.await_deferred() {
                                pending.await;
                            }
                            let reply = owner
                                .with(move || {
                                    let mut v = (app_fn)();
                                    v.dry_resolve();
                                    async move {
                                        //let v = v.resolve().await;
                                        let stream = set_meta
                                            .inject_meta_context(
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
                    move |path, _owner, content| {
                        let target = static_path(&leptos_options, Oco::Borrowed(path.as_ref()))
                            .expect("All paths are valid");
                        async move {
                            #[cfg(feature = "tokio")]
                            {
                                if let Some(parent) = target.parent() {
                                    tokio::fs::create_dir_all(parent).await?;
                                }
                                tokio::fs::write(target, content).await?;
                                Ok(())
                            }
                        }
                    }
                },
                |owner| false,
            )
            .await;
        Ok(())
    }
}
