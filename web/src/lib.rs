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

    #[cfg(any(feature = "debug-reload-panics", not(debug_assertions)))]
    {
        use crate::{
            helpers,
            local_storage::{
                LocalStorageAccessor, LocalStorageKey, get_current_local_storage_value,
                set_local_storage_value,
            },
        };
        pub use leptos::prelude::*;
        use std::{panic, sync::Once};
        struct LastPanic;
        struct LastPanicBuildNumber;
        const BUILD_NUMBER: Option<&str> = helpers::build_number();
        const RELOAD_PREFIX: &str = "reloaded";
        let reload_keyword = if let Some(number) = BUILD_NUMBER {
            Oco::Owned(format!("{RELOAD_PREFIX}-{number}"))
        } else {
            Oco::Borrowed(RELOAD_PREFIX)
        };
        impl LocalStorageAccessor for LastPanic {
            const KEY: LocalStorageKey = LocalStorageKey::LastPanic;
            type Data = String;
        }
        impl LocalStorageAccessor for LastPanicBuildNumber {
            const KEY: LocalStorageKey = LocalStorageKey::LastPanicBuildNumber;
            type Data = String;
        }
        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            let prev_hook = panic::take_hook();
            std::panic::set_hook(Box::new(move |info| {
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

                'reload_page: {
                    if let Some(location) = document().location() {
                        'reload_with_hash: {
                            let stored_build_number =
                                get_current_local_storage_value::<LastPanicBuildNumber>()
                                    .ok()
                                    .flatten();

                            if let Some(build_number) = BUILD_NUMBER
                                && stored_build_number.as_deref() != Some(build_number)
                                && set_local_storage_value::<LastPanicBuildNumber>(
                                    build_number.to_string(),
                                )
                                .is_ok()
                            {
                                leptos::logging::log!("First panic attack is free");
                                break 'reload_with_hash;
                            }

                            if let Ok(hash) = location.hash() {
                                if hash.strip_prefix('#').unwrap_or(&hash) == reload_keyword {
                                    break 'reload_page;
                                } else {
                                    leptos::logging::warn!(
                                        "It's likely the backend has updated, trying to reload"
                                    );
                                    _ = location.set_hash(&reload_keyword);
                                }
                            }
                        }
                        _ = location.reload_with_forceget(true);
                    }
                }
                prev_hook(info);
            }))
        });
    }
    leptos::mount::hydrate_lazy(App);
}
