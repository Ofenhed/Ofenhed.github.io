#![allow(unused)] // for now
use leptos::prelude::*;

#[component]
pub fn YouTube(
    #[prop(into)] id: Signal<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        // <iframe src=move || id.with(|id| format!("https://www.youtube-nocookie.com/embed/{id}")) sandbox="allow-scripts" allow="fullscreen; encrypted-media"></iframe>
        <a href=move || {
            id.with(|id| format!("https://youtube.com/watch?v={id}"))
        }>{children.map(|x| x())}</a>
    }
}
