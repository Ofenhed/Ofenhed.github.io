use leptos::{attr::custom::custom_attribute, prelude::*};
use leptos_meta::{Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    SsrMode,
    components::{A, Route, Router, Routes},
    hooks::use_location,
    path,
    static_routes::StaticRoute,
};

use crate::{
    blog::Blog,
    contact::{Contact, QrLogo},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    provide_meta_context();
    //let css_path = format!("/{}/{}.css", options.site_pkg_dir, options.output_name);
    let css_path = options.css_path();
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options />
                <Stylesheet href=css_path />
                <MetaTags/>
            </head>
            <body>
                <App/>
                <script>
                    addEventListener("DOMContentLoaded", (event) => {
                        const has_wasm = (() => {
                            try {
                                if (typeof WebAssembly === "object"
                                    && typeof WebAssembly.instantiate === "function") {
                                    const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
                                    if (module instanceof WebAssembly.Module)
                                        return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
                                }
                            } catch (e) {
                            }
                            return false;
                        })();
                        if (!has_wasm) {
                            document.querySelectorAll("img[wasm-fallback-src]").forEach((img) => {
                                img.setAttribute("src", img.getAttribute("wasm-fallback-src"));
                                img.removeAttribute("wasm-fallback-src");
                            });
                        }
                    });
                </script>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    let fallback = || view! { "Something hilarious about monkeys" }.into_view();
    {
        let (get_logo_status, save_logo_status) = signal(QrLogo::Hidden);
        provide_context(get_logo_status);
        provide_context(save_logo_status);
    }
    view! {
        <Title text="Condition Raise" />
        <Meta name="color-scheme" content="dark light" />
        <Router>
            <nav id="navigation">
                <input type="checkbox" id="hamburger-toggle" aria-label="hamburger" aria-controls="menu" aria-expanded="false" />
                <label for="hamburger-toggle" id="hamburger" aria-hidden="true">
                  <span class="slice" />
                  <span class="slice" />
                  <span class="slice" />
                </label>
                <ul id="menu" aria-hidden="true">
                  <li>
                    <A href="/">"Contact"</A>
                  </li>
                  <li>
                    <A href="/clog">"Clog"</A>
                  </li>
                </ul>
            </nav>
            <main {..custom_attribute("path", use_location().pathname)}>
            <Routes fallback>
                <Route
                  path=path!("/")
                  view=Contact
                  ssr=SsrMode::Static(StaticRoute::new())
                  />
                <Blog />
            </Routes>
            </main>
        </Router>
    }
}
