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

    //#[cfg(not(debug_assertions))]
    {
        use crate::{
            helpers::build_number,
            local_storage::{
                LocalStorageAccessor, LocalStorageKey, get_current_local_storage_value,
                set_local_storage_value,
            },
        };
        pub use leptos::prelude::*;
        use std::{panic, sync::Once};
        struct LastPanic;
        struct LastPanicBuildNumber;
        const RELOAD_KEYWORD: &str = "reloaded";
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
                if let Some(location) = document().location()
                    && let Ok(hash) = location.hash()
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
                    loop {
                        let build_number = build_number();
                        let stored_build_number =
                            get_current_local_storage_value::<LastPanicBuildNumber>();

                        if let Some(last_build) = stored_build_number.ok().flatten()
                            && Some(last_build.as_str())
                                == build_number.as_ref().map(|x| x.as_str())
                        {
                            leptos::logging::error!(
                                "Reload did not help with the crash, refusing to reload again"
                            );
                            break;
                        } else if let Some(build_number) = build_number
                            && set_local_storage_value::<LastPanicBuildNumber>(
                                build_number.to_string(),
                            )
                            .is_ok()
                        {
                        } else if hash != RELOAD_KEYWORD {
                            _ = location.set_hash(RELOAD_KEYWORD);
                        }
                        _ = location.reload_with_forceget(true);
                        break;
                    }
                }
                prev_hook(info);
            }))
        });
    }
    leptos::mount::hydrate_lazy(App);
}
