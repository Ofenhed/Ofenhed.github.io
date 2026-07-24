use leptos::{logging::error, prelude::*, task};
use leptos_router::{LazyRoute, lazy_route};

use crate::helpers::{NoScript, NoWasm};
use crate::local_storage::{
    LocalStorageAccessor, LocalStorageKey, get_local_storage_value, set_local_storage_value,
};
use crate::third_party::YoutubeConsentType;

impl LocalStorageAccessor for YoutubeConsentType {
    const KEY: LocalStorageKey = LocalStorageKey::YoutubeCookieConsent;

    type Data = YoutubeConsentType;
}

pub(crate) struct CookieConsent;

#[derive(Clone)]
struct ShowCookieConsentLink(Signal<bool>);
impl LocalStorageAccessor for ShowCookieConsentLink {
    const KEY: LocalStorageKey = LocalStorageKey::WantsCookies;
    type Data = bool;
}

#[derive(Clone)]
struct CookieConsentPageLoaded(WriteSignal<bool>);

fn show_cookie_consent_page() {
    if let Some(CookieConsentPageLoaded(loaded)) = use_context() {
        loaded.set(true);
    }
}

pub(crate) fn request_third_party_cookies() {
    let ShowCookieConsentLink(signal) = use_context().unwrap();
    if !signal.get_untracked() {
        task::spawn(CookieConsent::preload());
    }
    _ = set_local_storage_value::<ShowCookieConsentLink>(true);
    show_cookie_consent_page();
}

pub(crate) fn provide_cookie_consent_context() {
    let requested = get_local_storage_value::<ShowCookieConsentLink>().ok();

    let (cookie_page_loaded, set_cookie_page_loaded) = signal(false);
    provide_context(CookieConsentPageLoaded(set_cookie_page_loaded));

    provide_context(ShowCookieConsentLink(Signal::derive(move || {
        requested.and_then(|x| x.get()).unwrap_or(false) || cookie_page_loaded.get()
    })));
}

pub(crate) fn should_show_cookie_consent_link() -> Signal<bool> {
    let ShowCookieConsentLink(signal) = use_context().unwrap();
    signal
}

#[lazy_route]
impl LazyRoute for CookieConsent {
    fn data() -> Self {
        Self
    }

    fn view(_this: Self) -> AnyView {
        show_cookie_consent_page();
        let no_script = || {
            view! {
                <p>
                    "Cookies and tracking can not be configured (and are not used) if you don't have WASM support enabled."
                </p>
                <style nonce=use_nonce>"form.cookie-consent{display:none}"</style>
            }
        };
        view! {
            <h1>Cookies and tracking consent</h1>
            <NoScript>{no_script}</NoScript>
            <NoWasm>{no_script}</NoWasm>
            <form class:cookie-consent>
                <p>
                    "This is the third party tracking and cookies used by this web page. All cookies and tracking are opt in, meaning that no choice means no tracking (possibly excluding tracking functionality out of my control by the hosting provider)."
                </p>
                <fieldset>
                    <legend class:with-youtube-logo=true>YouTube</legend>
                    <YoutubeConsent />
                </fieldset>
                <div class:box=true class:thought=true>
                    "Speaking of consent, what's the opposite of "
                    <q>"legitimate interest"</q>
                    "?"
                </div>
            </form>
        }
        .into_any()
    }
}

#[component]
pub(crate) fn YoutubeConsent() -> impl IntoView {
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
    view! {
                <fieldset class:youtube=true>
                    <legend>No tracking or cookies</legend>
                    <label tabindex="0">
                        <input
                            type="radio"
                            name="youtube-tracking"
                            checked=current_content(YoutubeConsentType::PlainLink)
                            on:change=|e| {
                                if event_target_checked(&e)
                                    && let Err(e) = set_local_storage_value::<
                                        YoutubeConsentType,
                                    >(YoutubeConsentType::PlainLink)
                                {
                                    error!("Could not save consent data: {e}");
                                }
                            }
                        />
                        "A regular link to youtube"
                    </label>
                    <div class:box=true class:info=true>
                        "It does have a thumbnail, but you will not download it from YouTube. This also requests than no referrer information is sent to YouTube if you click on a video."
                    </div>
                </fieldset>
                <fieldset>
                    <legend>"Data is shared with YouTube"</legend>
                    <label tabindex="0">
                        <input
                            type="radio"
                            name="youtube-tracking"
                            checked=current_content(YoutubeConsentType::NoCookieDomain)
                            on:change=|e| {
                                if event_target_checked(&e)
                                    && let Err(e) = set_local_storage_value::<
                                        YoutubeConsentType,
                                    >(YoutubeConsentType::NoCookieDomain)
                                {
                                    error!("Could not save consent data: {e}");
                                }
                            }
                        />
                        "Embedded with reduced cookies (using "<q class:nobrplz=true>
                            "youtube-nocookies.com"
                        </q>
                        ")."
                    </label>
                    <label tabindex="0">
                        <input
                            type="radio"
                            name="youtube-tracking"
                            checked=current_content(YoutubeConsentType::RegularYoutube)
                            on:change=|e| {
                                if event_target_checked(&e)
                                    && let Err(e) = set_local_storage_value::<
                                        YoutubeConsentType,
                                    >(YoutubeConsentType::RegularYoutube)
                                {
                                    error!("Could not save consent data: {e}");
                                }
                            }
                        />
                        "Embedded without any cookie or tracking reduction"
                    </label>
                    <div class:box=true class:info=true>
                        "These options will send data to YouTube each time you visit a page "
                        <i>"with an embedded video"</i>
                        ", "
                        <b>"even without you even playing any videos"</b>
                        ". It could allow YouTube to track which pages you are reading (even thoug the referrer sent to YouTube won't show which page you're visiting, but it can be deduced). The reduced cookies version will still store data in your local storage, and might still deploy tracking cookies; The intent is to embed videos without affecting your watch history or recommendations, see "<a href="https://support.google.com/youtube/answer/171780?hl=en#zippy=%2Cturn-on-privacy-enhanced-mode">"Google Support for "<q class:nobrplz=true>"youtube-nocookies.com"</q></a>". It comes with the drawback that YouTube actively blocks unathenticated users from using a VPN, but that's an issue you can ignore if it's not causing you issues."
                    </div>

                </fieldset>
    }
}
