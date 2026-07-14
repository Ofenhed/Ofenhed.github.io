use std::{
    borrow::Cow,
    path::{Path, PathBuf},
    sync::Arc,
};

use any_spawner::PinnedFuture;
use futures::StreamExt as _;
use leptos::{config::LeptosOptions, prelude::*};
use leptos_integration_utils::{BoxedFnOnce, PinnedStream};
use leptos_router::{
    Method, PathSegment, RouteList, RouteListing, SsrMode, static_routes::StaticRoute,
};

pub mod prelude {
    pub use super::{
        NoGenerateStatic, StaticFileGeneratorError, StaticRouteGenerator, add_404,
        generate_static_files_list,
    };
}

pub struct StaticFileOptions<'a, C> {
    pub leptos: Oco<'a, LeptosOptions>,
    pub additional_context: C,
    pub excluded_routes: Vec<String>,
}

#[derive(Clone)]
pub struct NoGenerateStatic(pub WriteSignal<bool>);

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
            !matches!(x, C::Normal(_) | C::CurDir)
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

pub type StaticRouteGeneratorFunction = Box<dyn FnOnce(&LeptosOptions) -> PinnedFuture<()> + Send>;
pub struct StaticRouteGenerator(#[allow(unused)] Owner, StaticRouteGeneratorFunction);

#[derive(Clone)]
struct NoGenerateStaticReader(ReadSignal<bool>);

fn async_stream_builder<IV>(
    app: IV,
    chunks: BoxedFnOnce<PinnedStream<String>>,
    _supports_ooo: bool,
) -> PinnedFuture<PinnedStream<String>>
where
    IV: IntoView + 'static,
{
    Box::pin(async move {
        let app = app.to_html_stream_in_order();
        let app = app.collect::<String>().await;
        let chunks = chunks();
        Box::pin(futures::stream::once(async move { app }).chain(chunks)) as PinnedStream<String>
    })
}

impl StaticRouteGenerator {
    pub fn render_route<IV: IntoView + 'static>(
        path: Oco<'static, str>,
        app_fn: impl Fn() -> IV + Clone + Send + 'static,
        additional_context: impl Fn() + Clone + Send + 'static,
    ) -> impl Future<Output = (Owner, String)> {
        #[cfg(feature = "meta")]
        let (meta_context, meta_output) = leptos_meta::ServerMetaContext::new();
        let additional_context = {
            move || {
                provide_context(leptos_router::location::RequestUrl::new(path.as_ref()));
                #[cfg(feature = "meta")]
                {
                    provide_context(meta_context);
                }
                additional_context();
            }
        };
        let (owner, stream) = leptos_integration_utils::build_response(
            app_fn.clone(),
            additional_context,
            async_stream_builder,
            false,
        );

        let sc = owner.shared_context().unwrap();

        async move {
            let stream = stream.await;
            while let Some(pending) = sc.await_deferred() {
                pending.await;
            }

            #[cfg(feature = "meta")]
            let stream = meta_output.inject_meta_context(stream).await;
            let html = stream.collect::<String>().await;
            (owner, html)
        }
    }

    pub fn new<IV>(
        routes: &RouteList,
        app_fn: impl Fn() -> IV + Clone + Send + 'static,
        additional_context: impl Fn() + Clone + Send + 'static,
    ) -> Self
    where
        IV: IntoView + 'static,
    {
        let owner = Owner::new();
        Self(owner.clone(), {
            let routes = routes.clone();
            Box::new(move |options| {
                let options = options.clone();
                let app_fn = app_fn.clone();
                let additional_context = additional_context.clone();
                let additional_context = {
                    let options = options.clone();
                    move || {
                        let (read_ignored, write_ignored) = signal(false);
                        provide_context(NoGenerateStatic(write_ignored));
                        provide_context(NoGenerateStaticReader(read_ignored));
                        provide_context(options.clone());
                        additional_context();
                    }
                };

                owner.with(|| {
                    additional_context();
                    Box::pin(ScopedFuture::new(routes.generate_static_files(
                        move |path| {
                            eprintln!("Generating {path}");
                            Self::render_route(
                                Oco::Owned(path.to_string()),
                                app_fn.clone(),
                                additional_context.clone(),
                            )
                        },
                        move |path, owner, content| {
                            let target = static_path(&options, Oco::Borrowed(path.as_ref()))
                                .map_err(std::io::Error::other);
                            let path = path.to_owned();
                            let dont_generate = owner.with(|| {
                                let NoGenerateStaticReader(r) =
                                    use_context().expect("Inserted above");
                                r.get()
                            });

                            async move {
                                if dont_generate {
                                    eprintln!("Ignoring path {path}");
                                    return Ok(());
                                }
                                let target = target?;
                                eprintln!("Saving {path} as {}", target.display());
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
                        },
                        |_owner| false,
                    )))
                })
            })
        })
    }

    pub async fn generate(self, options: &LeptosOptions) {
        (self.1)(options).await
    }
}

pub fn add_404(list: &mut RouteList, options: &LeptosOptions) {
    let not_found_path = options.not_found_path.clone();
    let route_404 = RouteListing::new(
        [PathSegment::Static(Cow::Owned(not_found_path.to_string()))],
        SsrMode::Static(StaticRoute::new()),
        [Method::Get],
        [],
    );
    if list.iter().find(|x| x.path() == route_404.path()).is_none() {
        list.push(route_404);
    }
}

pub async fn generate_static_files_list<IV: IntoView + 'static>(
    app_fn: impl Clone + Send + 'static + Fn() -> IV,
    additional_context: impl Clone + Send + 'static + Fn(),
) -> Result<RouteList, StaticFileGeneratorError> {
    init_executor()?;
    let _owner = Owner::new_root(Some(Arc::from(hydration_context::SsrSharedContext::new())));
    provide_context(leptos_router::location::RequestUrl::new(""));
    #[cfg(feature = "meta")]
    {
        use leptos_meta::*;
        let (mock_meta, _) = ServerMetaContext::new();
        provide_context(mock_meta);
    }
    let (_, mock_writer) = signal(false);
    provide_context(NoGenerateStatic(mock_writer));
    additional_context();
    leptos_router::RouteList::generate(app_fn).ok_or(StaticFileGeneratorError::NoRoutesGenerated)
}
