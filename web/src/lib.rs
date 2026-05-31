pub mod app;
pub mod blog;
pub mod contact;
pub mod helpers;

#[cfg(feature = "nonce")]
pub fn maybe_nonce() -> Option<leptos::nonce::Nonce> {
    leptos::nonce::use_nonce()
}
#[cfg(not(feature = "nonce"))]
pub fn maybe_nonce() -> () {}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
