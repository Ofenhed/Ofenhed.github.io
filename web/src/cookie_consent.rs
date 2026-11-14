use leptos::{logging::error, prelude::*};
use leptos_router::{LazyRoute, lazy_route};

use crate::helpers::{Abbr, BoxType, NoScript, NoWasm, during_hydration, prefix_symbol};
use crate::local_storage::{
    LocalStorageAccessor, LocalStorageKey, get_local_storage_value, set_local_storage_value,
};
use crate::third_party::YoutubeConsentType;

impl LocalStorageAccessor for YoutubeConsentType {
    const KEY: LocalStorageKey = LocalStorageKey::YoutubeCookieConsent;

    type Data = YoutubeConsentType;
}

#[derive(Clone, Copy)]
pub(crate) struct CookieConsent(YoutubeConsent);

#[derive(Clone)]
struct ShowCookieConsentLink(Signal<bool>);
impl LocalStorageAccessor for ShowCookieConsentLink {
    const KEY: LocalStorageKey = LocalStorageKey::WantsCookies;
    type Data = bool;
}

#[derive(Clone)]
struct CookieConsentPageLoaded(WriteSignal<bool>);

pub(crate) fn request_third_party_cookies() {
    #[cfg(feature = "client-side")]
    {
        let ShowCookieConsentLink(signal) = use_context().unwrap();
        if !signal.get_untracked() {
            leptos::task::spawn_local(CookieConsent::preload());
        }
        _ = set_local_storage_value::<ShowCookieConsentLink>(true);
    }
}

pub(crate) fn provide_cookie_consent_context() {
    let requested = get_local_storage_value::<ShowCookieConsentLink>().ok();
    let during_hydration = during_hydration();

    let (cookie_page_loaded, set_cookie_page_loaded) = signal(false);
    provide_context(CookieConsentPageLoaded(set_cookie_page_loaded));

    provide_context(ShowCookieConsentLink(Signal::derive(move || {
        if cookie_page_loaded.get() {
            return true;
        }
        requested.and_then(|x| x.get()).unwrap_or(false) && !during_hydration.get()
    })));
}

pub(crate) fn should_show_cookie_consent_link() -> Signal<bool> {
    let ShowCookieConsentLink(signal) = use_context().unwrap();
    signal
}

#[lazy_route]
impl LazyRoute for CookieConsent {
    fn data() -> Self {
        Self(YoutubeConsent::data())
    }

    fn preload() -> impl Future<Output = ()> {
        YoutubeConsent::preload()
    }

    fn view(_this: Self) -> AnyView {
        if let Some(CookieConsentPageLoaded(is_cookie_consent)) = use_context() {
            is_cookie_consent.set(true);
            Owner::on_cleanup(move || {
                is_cookie_consent.set(false);
            });
        }
        let no_script = || {
            view! {
                <p>
                    "Cookies and tracking can not be configured (and are not used) if you don't have "
                    <Abbr no_expand=true title=Oco::Borrowed("WebAssembly")>
                        "Wasm"
                    </Abbr>" support enabled."
                </p>
                <style nonce=use_nonce>"form.cookie-consent{display:none}"</style>
            }
        };
        view! {
            <h1>Cookies and tracking consent</h1>
            <NoScript>{no_script}</NoScript>
            <NoWasm>{no_script}</NoWasm>
            <form class:cookie-consent=true>
                <p>
                    "This is the third party tracking and cookies used by this web page. All cookies and tracking are opt in, meaning that no choice means no tracking (possibly excluding tracking functionality out of my control by the hosting provider)."
                </p>
                <fieldset>
                    <legend class:with-youtube-logo=true>YouTube</legend>
                    {move || Suspend::new(YoutubeConsent::view(_this.0))}
                </fieldset>
                <div {..BoxType::Thought
                    .attr()}>
                    "Speaking of consent, what's the opposite of " <q>"legitimate interest"</q> "?"
                </div>
            </form>
        }
        .into_any()
    }
}

#[derive(Clone, Copy)]
pub(crate) struct YoutubeConsent;

#[lazy_route]
impl LazyRoute for YoutubeConsent {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        let youtube_consent = get_local_storage_value::<YoutubeConsentType>()
            .map_err(|err| {
                leptos::logging::warn!("Local storage error: {err}");
                err
            })
            .ok();
        let current_content = |value: YoutubeConsentType| {
            move || {
                if let Some(current) = youtube_consent.and_then(|x| x.get()) {
                    value == current
                } else {
                    false
                }
            }
        };
        let update_value = move |consent: YoutubeConsentType| {
            if youtube_consent.and_then(|x| x.get_untracked()) != Some(consent) {
                set_local_storage_value::<YoutubeConsentType>(consent)
            } else {
                Ok(())
            }
        };
        view! {
            <fieldset class:youtube=true>
                <legend {..prefix_symbol('\u{1F576}')}>"No tracking or cookies"</legend>
                <label tabindex="0">
                    <input
                        type="radio"
                        name="youtube-tracking"
                        prop:checked=current_content(YoutubeConsentType::PlainLink)
                        on:change=move |e| {
                            if event_target_checked(&e)
                                && let Err(e) = update_value(YoutubeConsentType::PlainLink)
                            {
                                error!("Could not save consent data: {e}");
                            }
                        }
                    />
                    "A regular link to youtube"
                </label>
                <div {..BoxType::Info
                    .attr()}>
                    "It does have a thumbnail, but you will not download it from YouTube. This also requests than no referrer information is sent to YouTube if you click on a video."
                </div>
            </fieldset>
            <fieldset>
                <legend {..prefix_symbol('\u{1F575}')}>"Data is shared with YouTube"</legend>
                <label tabindex="0">
                    <input
                        type="radio"
                        name="youtube-tracking"
                        prop:checked=current_content(YoutubeConsentType::NoCookieDomain)
                        on:change=move |e| {
                            if event_target_checked(&e)
                                && let Err(e) = update_value(YoutubeConsentType::NoCookieDomain)
                            {
                                error!("Could not save consent data: {e}");
                            }
                        }
                    />
                    "Embedded with reduced cookies (using "
                    <q class:nobrplz=true>"youtube-nocookies.com"</q>
                    ")."
                </label>
                <label tabindex="0">
                    <input
                        type="radio"
                        name="youtube-tracking"
                        prop:checked=current_content(YoutubeConsentType::RegularYoutube)
                        on:change=move |e| {
                            if event_target_checked(&e)
                                && let Err(e) = update_value(YoutubeConsentType::RegularYoutube)
                            {
                                error!("Could not save consent data: {e}");
                            }
                        }
                    />
                    "Embedded (no reduction of cookies or other tracking)"
                </label>
                <div {..BoxType::Info
                    .attr()}>
                    "These options will send data to YouTube each time you visit a page "
                    <i>"with an embedded video"</i> ", "
                    <b>"even without you playing any videos"</b>
                    ". It could allow YouTube to track which pages you are reading (even though the referrer sent to YouTube won't show which page you're visiting, but it can be deduced). The reduced cookies version will still store data in your local storage, and might still deploy tracking cookies; The intent is to embed videos without affecting your watch history or recommendations, see "
                    <a
                        href="https://support.google.com/youtube/answer/171780?hl=en#zippy=%2Cturn-on-privacy-enhanced-mode"
                        tabindex="-1"
                    >
                        "Google Support for "
                        <q class:nobrplz=true>"youtube-nocookies.com"</q>
                    </a>
                    ". It comes with the drawback that YouTube actively blocks unathenticated users from using a VPN, but that's an issue you can ignore if it's not causing you issues."
                </div>

            </fieldset>
        }.into_any()
    }
}
