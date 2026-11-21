#![recursion_limit = "256"]

pub(crate) mod app;
pub(crate) mod blog;
pub(crate) mod contact;
pub(crate) mod cookie_consent;
pub(crate) mod error;
pub(crate) mod helpers;
pub(crate) mod local_storage;
pub(crate) mod third_party;

pub(crate) const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");

#[cfg(feature = "ssr")]
pub use contact::qr_generator::save_qrcode;

pub use app::shell;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    use leptos::prelude::Notify;

    #[cfg(feature = "console-panic")]
    {
        console_error_panic_hook::set_once();
    }

    _ = any_spawner::Executor::init_wasm_bindgen();
    leptos::task::spawn_local(async move {
        let owner = leptos::mount::hydrate_from_async(leptos::tachys::dom::body(), App).await;
        if let (_, Some(signal)) = crate::helpers::during_hydration_internal() {
            signal.notify();
        }
        owner.forget();
    });
}
