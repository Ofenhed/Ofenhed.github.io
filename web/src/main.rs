#![recursion_limit = "256"]

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use conditionraise::{save_qrcode, shell};
    use leptos::{logging::log, prelude::*};
    use tokio::{pin, select};

    let conf = get_configuration(None).unwrap();
    #[cfg(feature = "dev")]
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let qr_options = leptos_options.clone();
    let qr = tokio::task::spawn_blocking(move || save_qrcode(&qr_options));
    pin!(qr);
    #[cfg(feature = "dev")]
    {
        let (routes, static_routes) = leptos_axum::generate_route_list_with_ssg({
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        });

        let do_generate = static_routes.generate(&leptos_options);
        pin!(do_generate);
        let [mut routes_saved, mut qr_saved] = [false; 2];
        loop {
            select! {
                q = &mut qr, if !qr_saved => {
                    _ = q.expect("qr code can always be constructed");
                    qr_saved = true;
                }
                _ = &mut do_generate, if !routes_saved => {
                    routes_saved = true;
                }
                else => {
                    break;
                }
            }
        }
        log!("Static files generated");

        use axum::Router;
        use leptos_axum::LeptosRoutes;
        let app = Router::new()
            .leptos_routes(&leptos_options, routes, {
                let leptos_options = leptos_options.clone();
                move || shell(leptos_options.clone())
            })
            .fallback(leptos_axum::file_and_error_handler(shell))
            .with_state(leptos_options.clone());

        // run our app with hyper
        // `axum::Server` is a re-export of `hyper::Server`
        log!("listening on http://{}", &addr);
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        axum::serve(listener, app.into_make_service())
            .await
            .unwrap();
    };

    #[cfg(feature = "statics")]
    {
        use leptos_static_files::prelude::*;
        let app = {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        };
        let mut routes = generate_static_files_list(app.clone(), || ())
            .await
            .unwrap();
        add_404(&mut routes, &leptos_options);
        let generator = StaticRouteGenerator::new(&routes, app, || ());
        let do_generate = generator.generate(&leptos_options);
        pin!(do_generate);
        let [mut routes_saved, mut qr_saved] = [false; 2];
        loop {
            select! {
                q = &mut qr, if !qr_saved => {
                    _ = q.expect("qr code can always be constructed");
                    qr_saved = true;
                }
                () = &mut do_generate, if !routes_saved => {
                    routes_saved = true;
                }
                else => {
                    break;
                }
            }
        }
        log!("Static files generated");
    }
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
