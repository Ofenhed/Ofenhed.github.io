use leptos::{attr::custom::custom_attribute, html, prelude::*};
use leptos_meta::{Meta, MetaTags, Title, provide_meta_context};
use leptos_router::{
    SsrMode,
    components::{A, Route, Router, Routes},
    hooks::use_location,
    path,
    static_routes::StaticRoute,
};

use crate::{
    blog::Blog,
    contact::{Contact, PersistentQrLogo},
    helpers::{Footnotes, ImgDef},
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    let css_path = options.css_path();
    view! {
        <!DOCTYPE html>
        <html lang="sv">
            <head>
                <link rel="stylesheet" href=css_path />
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="referrer" content="no-referrer" />
                <meta name="format-detection" content="telephone=no" />
                <meta name="format-detection" content="date=no" />
                <meta name="format-detection" content="address=no" />
                <meta name="format-detection" content="email=no" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
                <NoInitTransition />
            </body>
        </html>
    }
}

#[component]
pub(crate) fn BuildInfo() -> impl IntoView {
    use std::option_env;
    let data: [(Oco<'static, str>, Option<Oco<'static, str>>); _] = [
        (
            Oco::Borrowed("Commit"),
            option_env!("GITHUB_SHA").map(Oco::Borrowed),
        ),
        (
            Oco::Borrowed("Run number"),
            option_env!("GITHUB_RUN_NUMBER").map(Oco::Borrowed),
        ),
        (
            Oco::Borrowed("Build OS"),
            option_env!("RUNNER_OS").map(Oco::Borrowed),
        ),
    ];
    let server_url = if let (Some(url), Some(repo), Some(run_id)) = (
        option_env!("GITHUB_SERVER_URL"),
        option_env!("GITHUB_REPOSITORY"),
        option_env!("GITHUB_RUN_ID"),
    ) {
        Some(view! {
            <a href=format!("{url}/{repo}/actions/runs/{run_id}")>
                <img
                    {..ImgDef()}
                    src=format!("{url}/{repo}/actions/workflows/publish.yml/badge.svg")
                />
            </a>
        })
    } else {
        None
    };
    view! {
        <For each=move || data.clone() key=|x| x.clone() let:(d)>
            <fieldset>
                <legend>{d.0}</legend>
                {d.1}
            </fieldset>
        </For>
        {server_url}
    }
}

#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(PartialEq, Eq, Clone, Copy)]
enum EggCounter {
    Counter(u8),
    TriggeredOnce(u8),
    Triggered,
}

#[component]
pub(crate) fn NotFound() -> impl IntoView {
    view! {
        <h1>"404 error"</h1>
        <p>"Something hilarious about trained monkeys. So relatable. No evil here."</p>
    }
}

#[derive(Clone)]
pub(crate) struct HamburgerMenu(pub NodeRef<html::Input>);

#[component]
pub(crate) fn NoInitTransition() -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let script = js_macro::minify_js! {
            addEventListener("DOMContentLoaded", (event) => {
                const job = () => {
                    document.body.setAttribute("data-activate-transitions", "");
                };
                if (window.requestIdleCallback) {
                    window.requestIdleCallback(job);
                } else {
                    setTimeout(job, 1000);
                }
            });
        };
        view! { <script>{script}</script> }
    }
}

#[component]
pub(crate) fn App() -> impl IntoView {
    provide_meta_context();
    let fallback = || NotFound().into_view();
    let hamburger_toggle = NodeRef::new();
    provide_context(HamburgerMenu(hamburger_toggle.clone()));
    const INITIAL_EGG_COUNTER: u8 = 8;
    let (clicks_to_easter, set_clicks_to_easter) = signal(EggCounter::Counter(INITIAL_EGG_COUNTER));
    let (aria_expanded, set_aria_expanded) = signal(false);
    let sub_egg_count = move || {
        let path = use_location().pathname;
        let e = Effect::new(move |_| {
            if !path.with(|path| path == "/") {
                clicks_to_easter.track();
                set_clicks_to_easter.update_untracked(|x| match x {
                    EggCounter::Counter(count) | EggCounter::TriggeredOnce(count) => {
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
                EggCounter::TriggeredOnce(0) => *c = EggCounter::Triggered,
                EggCounter::Counter(0) => *c = EggCounter::TriggeredOnce(INITIAL_EGG_COUNTER),
                EggCounter::Counter(x) | EggCounter::TriggeredOnce(x) => {
                    *x -= 1;
                }
                EggCounter::Triggered => (),
            });
        }
    };
    let (get_logo_status, save_logo_status) = signal(PersistentQrLogo::default());
    provide_context(get_logo_status);
    provide_context(save_logo_status);
    let aria_hidden = Signal::derive(move || if aria_expanded.get() { "false" } else { "true" });
    view! {
        <Title text="Condition Raise" />
        <Meta name="color-scheme" content="dark light" />
        <Router>
            <nav id="navigation">
                <input
                    node_ref=hamburger_toggle
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
                    class:inner-conflict=move || {
                        clicks_to_easter.with(|x| *x == EggCounter::Triggered)
                    }
                    class:ultra-realistic=move || {
                        clicks_to_easter.with(|x| !matches!(x, EggCounter::Counter(_)))
                    }
                >
                    <span class="slice" />
                    <span class="slice" />
                    <span class="slice" />
                </label>
                <menu id="menu" aria-hidden=aria_hidden>
                    <li>
                        <A href="/">"Contact"</A>
                    </li>
                    <li>
                        <A href="/clog">"Clog"</A>
                    </li>
                </menu>
            </nav>
            <main {..custom_attribute("data-path", use_location().pathname)}>
                <Routes fallback>
                    <Route path=path!("/") view=Contact ssr=SsrMode::Static(StaticRoute::new()) />
                    <Route
                        path=path!("/build")
                        view=BuildInfo
                        ssr=SsrMode::Static(StaticRoute::new())
                    />
                    <Blog />
                </Routes>
            </main>
            <Footnotes />
        </Router>
    }
}
