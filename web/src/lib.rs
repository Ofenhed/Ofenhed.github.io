#![recursion_limit = "256"]

pub(crate) mod app;
pub(crate) mod blog;
pub(crate) mod contact;
pub(crate) mod cookie_consent;
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
    console_error_panic_hook::set_once();

    #[cfg(not(debug_assertions))]
    {
        use crate::local_storage::{
            LocalStorageAccessor, LocalStorageKey, set_local_storage_value,
        };
        pub use leptos::prelude::*;
        use std::{panic, sync::Once, time::Duration};
        struct LastPanic;
        const RELOAD_KEYWORD: &str = "reloaded";
        impl LocalStorageAccessor for LastPanic {
            const KEY: LocalStorageKey = LocalStorageKey::LastPanic;
            type Data = String;
        }
        static SET_HOOK: Once = Once::new();
        set_timeout(
            || {
                SET_HOOK.call_once(|| {
                    let prev_hook = panic::take_hook();
                    std::panic::set_hook(Box::new(move |info| {
                        if let Some(location) = document().location()
                            && let Ok(hash) = location.hash()
                            && hash != RELOAD_KEYWORD
                            && location.set_hash(RELOAD_KEYWORD).is_ok()
                        {
                            let mut panic_msg = String::new();
                            if let Some(location) = info.location() {
                                panic_msg.push_str(&format!(
                                    "{}@{}:{}\n",
                                    location.file(),
                                    location.line(),
                                    location.column()
                                ));
                            }
                            if let Some(panic_info) = info.payload_as_str() {
                                panic_msg.push_str(panic_info);
                                _ = set_local_storage_value::<LastPanic>(panic_msg);
                            }
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
