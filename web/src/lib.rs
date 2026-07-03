pub(crate) mod app;
pub(crate) mod blog;
pub(crate) mod contact;
pub(crate) mod helpers;
pub(crate) mod third_party;

#[cfg(feature = "ssr")]
pub use contact::qr_generator::save_qrcode;

pub use app::shell;
pub use leptos::prelude::document;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();

    #[cfg(not(debug_assertions))]
    {
        use std::{panic, sync::Once};
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            let prev_hook = panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
                if let Some(location) = document().location() {
                    location.reload_with_forceget(true);
                }
                prev_hook(info);
            }))
        });
    }
    leptos::mount::hydrate_lazy(App);
}
