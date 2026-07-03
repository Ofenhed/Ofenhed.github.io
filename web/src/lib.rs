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

    #[cfg(not(debug_assertions))]
    {
        use std::{panic, sync::Once};
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            let prev_hook = panic::panic::take_hook();
            std::panic::panic::set_hook(Box::new(move |_| {
                if let Some(location) = document().location() {
                    location.reload_with_forceget(true);
                }
                prev_hook();
            }))
        });
    }
    leptos::mount::hydrate_lazy(App);
}
