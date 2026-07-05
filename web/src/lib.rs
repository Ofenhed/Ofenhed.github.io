#![recursion_limit = "256"]

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
        pub use leptos::prelude::*;
        use std::{panic, sync::Once, time::Duration};
        static SET_HOOK: Once = Once::new();
        set_timeout(
            || {
                SET_HOOK.call_once(|| {
                    let prev_hook = panic::take_hook();
                    std::panic::set_hook(Box::new(move |info| {
                        if let Some(location) = document().location()
                            && let Ok(hash) = location.hash()
                            && hash != "panic"
                            && location.set_hash("panic").is_ok()
                        {
                            _ = location.reload_with_forceget(true);
                        }
                        prev_hook(info);
                    }))
                })
            },
            Duration::from_secs(3),
        );
    }
    leptos::mount::hydrate_lazy(App);
}
