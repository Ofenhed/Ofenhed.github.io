#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use conditionraise::app::*;
    use leptos::{logging::log, prelude::*};
    use leptos_axum::generate_route_list_with_ssg;
    use tokio::{pin, select};

    let conf = get_configuration(None).unwrap();
    #[cfg(feature = "dev")]
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let (_routes, static_routes) = generate_route_list_with_ssg({
        let leptos_options = leptos_options.clone();
        move || shell(leptos_options.clone())
    });

    {
        let routes = static_routes.generate(&leptos_options);
        let qr_options = leptos_options.clone();
        let qr = tokio::task::spawn_blocking(move || {
            conditionraise::contact::qr_generator::save_qrcode(&qr_options)
        });
        pin!(routes);
        pin!(qr);
        let [mut routes_saved, mut qr_saved] = [false; 2];
        loop {
            select! {
                q = &mut qr, if !qr_saved => {
                    _ = q.expect("qr code can always be constructed");
                    qr_saved = true;
                }
                _ = &mut routes, if !routes_saved => {
                    routes_saved = true;
                }
                else => {
                    break;
                }
            }
        }
        log!("Static files generated");
    }

    #[cfg(feature = "dev")]
    {
        use axum::Router;
        use leptos_axum::LeptosRoutes;
        let app = Router::new()
            .leptos_routes(&leptos_options, _routes, {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            })
            .fallback(leptos_axum::file_and_error_handler(shell))
            .with_state(leptos_options);

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        log!("listening on http://{}", &addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
