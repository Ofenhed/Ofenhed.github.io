use crate::{
    helpers,
    local_storage::{
        LocalStorageAccessor, LocalStorageKey, get_local_storage_value, set_local_storage_value,
    },
};
pub use leptos::prelude::*;

#[component]
pub fn ReloadErrorHandler(children: TypedChildren<impl IntoView + 'static>) -> impl IntoView {
    struct LastPanicBuildNumber;
    const BUILD_NUMBER: Option<&str> = helpers::build_number();
    const RELOAD_PREFIX: &str = "reloaded";
    let reload_keyword = if let Some(number) = BUILD_NUMBER {
        Oco::Owned(format!("{RELOAD_PREFIX}-{number}"))
    } else {
        Oco::Borrowed(RELOAD_PREFIX)
    };
    impl LocalStorageAccessor for LastPanicBuildNumber {
        const KEY: LocalStorageKey = LocalStorageKey::LastPanicBuildNumber;
        type Data = String;
    }

    let on_error = move |_| {
        'reload_page: {
            if let Some(location) = document().location() {
                'reload_with_hash: {
                    let stored_build_number = get_local_storage_value::<LastPanicBuildNumber>()
                        .ok()
                        .and_then(|x| x.get_untracked());

                    if let Some(build_number) = BUILD_NUMBER
                        && stored_build_number.as_deref() != Some(build_number)
                        && set_local_storage_value::<LastPanicBuildNumber>(build_number.to_string())
                            .is_ok()
                    {
                        leptos::logging::log!("First panic attack is free");
                        break 'reload_with_hash;
                    }

                    if let Ok(hash) = location.hash() {
                        if hash.strip_prefix('#').unwrap_or(&hash) == reload_keyword {
                            leptos::logging::error!("Reloading did not solve the issue");
                            break 'reload_page;
                        } else {
                            leptos::logging::warn!(
                                "It's likely the backend has updated, trying to reload"
                            );
                            _ = location.set_hash(&reload_keyword);
                        }
                    }
                }
                leptos::logging::log!("Reloading");
                if let Err(e) = location
                    .reload_with_forceget(true)
                    .or_else(|_| location.reload())
                {
                    leptos::logging::error!(
                        "Failed to reload: {}",
                        e.as_string().unwrap_or("Unknown".to_string())
                    );
                } else {
                    if let Ok(l) = location.href() {
                        _ = location.replace(&l);
                    }
                }
            }
        }
        view! { "An unknown error occured" }
    };
    view! { <ErrorBoundary fallback=on_error>{(children.into_inner())()}</ErrorBoundary> }
}
