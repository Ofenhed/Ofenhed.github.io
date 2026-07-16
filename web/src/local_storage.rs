use std::str::FromStr;

use leptos::{
    ev::{self, StorageEvent},
    prelude::*,
};
use strum::{EnumString, IntoStaticStr, VariantArray};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub(crate) enum LocalStorageError {
    #[cfg_attr(not(feature = "client-side"), expect(unused))]
    #[error("Local storage not supported")]
    NotSupported,
    #[error("No local storage context provided")]
    NoContextProvided,
    #[error("Save failed")]
    SaveFailed,
}

#[derive(Clone, Copy, PartialEq, Eq, IntoStaticStr, VariantArray, EnumString)]
#[strum(serialize_all = "PascalCase")]
pub(crate) enum LocalStorageKey {
    WantsCookies,
    YoutubeCookieConsent,
}

pub(crate) trait LocalStorageData: 'static + Send + Sync + FromStr + ToString {}
impl<T: 'static + Send + Sync + FromStr + ToString> LocalStorageData for T {}

#[derive(Clone, Copy)]
struct LocalStorageChanged(ReadSignal<()>, WriteSignal<()>);

pub(crate) fn provide_local_storage_context() {
    let context @ LocalStorageChanged(_, writer) = {
        let (read, write) = signal(());
        LocalStorageChanged(read, write)
    };
    provide_context(context);
    window_event_listener(ev::storage, move |_: StorageEvent| {
        writer.notify();
    });
}

#[cfg(feature = "client-side")]
fn local_storage() -> Result<web_sys::Storage, LocalStorageError> {
    if let Ok(Some(storage)) = window().local_storage() {
        Ok(storage)
    } else {
        Err(LocalStorageError::NotSupported)
    }
}

pub(crate) fn get_local_storage_value<T: LocalStorageAccessor>()
-> Result<Signal<Option<T::Data>>, LocalStorageError> {
    let LocalStorageChanged(reader, _) =
        use_context().ok_or(LocalStorageError::NoContextProvided)?;
    let (read, write) = signal(None);
    cfg_select! {
        feature = "client-side" =>
    {
        let storage = local_storage()?;
        Effect::new(move || {
            reader.track();
            if let Some(value) = storage
                .get_item(crate::helpers::into_static_str(T::KEY))
                .expect("This should always work if we get this far")
                .and_then(|x|
                    <T::Data as FromStr>::from_str(x.as_str())
                        .inspect_err(|_| leptos::logging::error!("Could not parse {x}"))
                        .ok()
                )
            {
                write.set(Some(value));
            } else {
                let mut w = write.write();
                if w.is_some() {
                    *w = None;
                } else {
                    w.untrack();
                }
            }
        });
    }
        _ => {
            _ = (reader, write, LocalStorageError::NoContextProvided);
        }
    }
    Ok(read.into())
}

pub(crate) fn set_local_storage_value<T: LocalStorageAccessor>(
    value: T::Data,
) -> Result<(), LocalStorageError> {
    cfg_select! {
        feature = "client-side" =>
    {
        let storage = local_storage()?;
        let LocalStorageChanged(_, writer) =
            use_context().ok_or(LocalStorageError::NoContextProvided)?;
        storage
            .set_item(crate::helpers::into_static_str(T::KEY), &value.to_string())
            .map_err(|_| LocalStorageError::SaveFailed)?;
        writer.notify();
    }
        _ => {
            _ = (value, T::KEY, LocalStorageError::SaveFailed);
        }
    }
    Ok(())
}

pub(crate) trait LocalStorageAccessor {
    const KEY: LocalStorageKey;
    type Data: LocalStorageData;
}
