use leptos::{attr::custom::custom_attribute, prelude::*};
use leptos_meta::{Meta, MetaTags, Script, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    SsrMode,
    components::{A, Route, Router, Routes},
    hooks::use_location,
    path,
    static_routes::StaticRoute,
};

use crate::{
    blog::Blog,
    contact::{AnimateQrLogo, Contact},
};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct ShowNavigation(pub bool);

pub fn shell(options: LeptosOptions) -> impl IntoView {
    //let css_path = format!("/{}/{}.css", options.site_pkg_dir, options.output_name);
    let css_path = options.css_path();
    let minified_js = js_macro::minify_js! {
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
    };
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <Stylesheet href=css_path />
                <MetaTags />
            </head>
            <body>
                <App />
                <Script>{minified_js}</Script>
            </body>
        </html>
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
enum EggCounter {
    Counter(u8),
    Triggered,
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    let fallback = || {
        view! {
            <h1>"404 error"</h1>
            <p>"Something hilarious about trained monkeys. So relatable. No evil here."</p>
        }
        .into_view()
    };
    const INITIAL_EGG_COUNTER: u8 = 8;
    let (clicks_to_easter, set_clicks_to_easter) = signal(EggCounter::Counter(INITIAL_EGG_COUNTER));
    let (aria_expanded, set_aria_expanded) = signal(false);
    let sub_egg_count = move || {
        let path = use_location().pathname;
        let e = Effect::new(move |_| {
            if !path.with(|path| path == "/") {
                clicks_to_easter.track();
                set_clicks_to_easter.update_untracked(|x| match x {
                    EggCounter::Counter(count) => {
                        *count = INITIAL_EGG_COUNTER;
                    }
                    _ => (),
                });
            }
        });
        move |event: leptos::ev::Event| {
            let _e = e;
            set_aria_expanded.set(event_target_checked(&event));
            set_clicks_to_easter.update(move |c| match c {
                EggCounter::Counter(0) => *c = EggCounter::Triggered,
                EggCounter::Counter(x) => {
                    *x -= 1;
                }
                EggCounter::Triggered => (),
            });
        }
    };
    let (get_logo_status, save_logo_status) = signal(AnimateQrLogo(true));
    provide_context(get_logo_status);
    provide_context(save_logo_status);
    let show_navigation = {
        let (should_show_navigation, set_show_navigation) = signal(ShowNavigation(false));
        provide_context(set_show_navigation);
        Signal::derive(move || {
            if ShowNavigation(true) == should_show_navigation.get() {
                true
            } else {
                false
            }
        })
    };
    let aria_hidden = Signal::derive(move || if aria_expanded.get() { "false" } else { "true" });
    view! {
        <Title text="Condition Raise" />
        <Meta name="color-scheme" content="dark light" />
        <Router>
            <nav id="navigation" class:beta-nav=show_navigation>
                <input
                    type="checkbox"
                    id="hamburger-toggle"
                    aria-label="hamburger"
                    aria-controls="menu"
                    aria-expanded=Signal::derive(move || {
                        if aria_expanded.get() { "true" } else { "false" }
                    })
                    on:change=sub_egg_count()
                />
                <label
                    for="hamburger-toggle"
                    id="hamburger"
                    aria-hidden="true"
                    class:egg=move || clicks_to_easter.get() == EggCounter::Triggered
                >
                    <span class="slice" />
                    <span class="slice" />
                    <span class="slice" />
                </label>
                <ul id="menu" aria-hidden=aria_hidden>
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
                    <Route path=path!("/") view=Contact ssr=SsrMode::Static(StaticRoute::new()) />
                    <Blog />
                </Routes>
            </main>
        </Router>
    }
}
