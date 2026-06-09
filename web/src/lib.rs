pub(crate) mod app;
pub(crate) mod blog;
pub(crate) mod contact;
pub(crate) mod helpers;
pub(crate) mod third_party;

#[cfg(feature = "ssr")]
pub use contact::qr_generator::save_qrcode;

pub use app::shell;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
