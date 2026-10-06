use std::borrow::Cow;

use strum::{AsRefStr, EnumString, IntoStaticStr, VariantArray};

use leptos::{
    attr::{self, custom::custom_attribute},
    either::Either,
    ev, html,
    prelude::*,
};

use crate::{
    cookie_consent::{YoutubeConsent, request_third_party_cookies},
    helpers::{has_interested_owners, once_by_type, register_interested_owner},
    local_storage::get_local_storage_value,
};
cfg_select! {
    feature = "client-side" => {
        use js_sys::JsString;
        use wasm_bindgen::intern;
    }
    _ => {}
}

#[derive(
    Default, Clone, Copy, PartialEq, Eq, AsRefStr, IntoStaticStr, VariantArray, EnumString,
)]
pub enum YoutubeConsentType {
    #[default]
    PlainLink,
    NoCookieDomain,
    RegularYoutube,
}

#[derive(Clone)]
pub struct YoutubeVideo {
    pub id: &'static str,
    pub title: Option<Cow<'static, str>>,
    pub author_url: Option<Cow<'static, str>>,
    pub author_name: Option<Cow<'static, str>>,
    pub width: usize,
    pub height: usize,
    #[cfg(feature = "ssr")]
    pub thumbnail_url: Option<Cow<'static, str>>,
}

macro_rules! youtube {
    ($id:literal) => {
        const {
            const DATA: oembed::OembedData = oembed::oembed! {
                    "https://www.youtube.com/oembed",
                    "https://www.youtube.com/watch?v=" + $id
            };
            const SIZE: (usize, usize) = match DATA.content {
                oembed::OembedType::Video { width, height, .. } => (width, height),
            };
            #[allow(unused)]
            const VIDEO: $crate::third_party::YoutubeVideo = $crate::third_party::YoutubeVideo {
                id: $id,
                title: DATA.title,
                author_url: DATA.author_url,
                author_name: DATA.author_name,
                #[cfg(feature = "ssr")]
                thumbnail_url: DATA.thumbnail_url,
                width: SIZE.0,
                height: SIZE.1,
            };
            VIDEO
        }
    };
    ($id:literal ($width:literal : $height:literal)) => {
        const {
            let mut yv = youtube!($id);
            (yv.width, yv.height) = ($width, $height);
            yv
        }
    };
}
pub(crate) use youtube;

impl std::fmt::Display for YoutubeConsentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_ref())
    }
}

#[cfg(feature = "ssr")]
mod downloader {
    use super::*;
    use reqwest::StatusCode;
    use std::{
        borrow::Cow,
        path::{Path, PathBuf},
    };
    use tokio::{fs::*, io::AsyncWriteExt};
    #[derive(thiserror::Error, Debug)]
    pub enum DownloadError {
        #[error(transparent)]
        Reqwest(#[from] reqwest::Error),
        #[error("Invalid HTTP response: {0}")]
        InvalidHttp(StatusCode),
        #[error(transparent)]
        Io(#[from] std::io::Error),
        #[cfg(feature = "ffmpeg")]
        #[error(transparent)]
        Join(#[from] tokio::task::JoinError),
        #[cfg(feature = "ffmpeg")]
        #[error("ImageMagick failed with code {0}")]
        ImageMagick(i32),
    }
    pub async fn download_youtube_thumbnail(video: &YoutubeVideo) -> Result<(), DownloadError> {
        let options = use_context::<LeptosOptions>()
            .expect("YouTube integration requires LeptosOptions as context");
        let cache_root = Path::new("target/youtube_thumbnails");
        let target_root = PathBuf::from(format!("{}/youtube", options.site_root));
        create_dir_all(&target_root).await?;
        create_dir_all(&cache_root).await?;
        let mut target_file = target_root.clone();
        let mut cache_file = cache_root.to_path_buf();
        for path in [&mut target_file, &mut cache_file] {
            path.push(format!("{}.jpg", video.id));
        }
        'download_image: {
            if let Ok(mut image_file) = File::create(&cache_file).await {
                let client = reqwest::Client::new();
                let video_id = video.id;

                let image_source = [
                    Cow::Owned(format!(
                        "https://i.ytimg.com/vi/{video_id}/maxresdefault.jpg"
                    )),
                    video.thumbnail_url.clone().unwrap_or_else(|| {
                        Cow::Owned(format!("https://i.ytimg.com/vi/{video_id}/hqdefault.jpg"))
                    }),
                ];
                let mut last_status = None;
                for source in image_source {
                    eprintln!("Downloading thumbnail for youtube/{video_id}");
                    let mut image = client.get(&*source).send().await?;
                    if image.status().is_success() {
                        while let Some(chunk) = image.chunk().await? {
                            image_file.write_all(&chunk).await?;
                        }
                        let mut perms = image_file.metadata().await?.permissions();
                        perms.set_readonly(true);
                        image_file.set_permissions(perms).await?;
                        break 'download_image;
                    } else {
                        last_status = Some(image.status())
                    }
                }
                return Err(DownloadError::InvalidHttp(last_status.unwrap()));
            }
        }
        {
            cfg_select! {
                feature = "ffmpeg" => {
                    use std::sync::Arc;
                    use tokio::{process::Command, task::JoinSet};
                    let mut set = JoinSet::new();
                    let cache_file = Arc::new(cache_file);
                    for ext in ["avif", "webp", "jpg"] {
                        let mut output_file = target_root.clone();
                        output_file.push(format!("{}.{ext}", video.id));
                        let cache_file = cache_file.clone();
                        let (w, h) = (video.width, video.height);
                        set.spawn(async move {
                            println!("Creating file {}", output_file.display());
                            Command::new("ffmpeg")
                                .arg("-i")
                                .arg(&*cache_file)
                                .arg("-y") // Allow overwrite
                                .arg("-loglevel")
                                .arg("warning")
                                .arg("-frames:v")
                                .arg("1")
                                .arg("-update")
                                .arg("1")
                                .arg("-vf")
                                .arg(format!("crop='min(iw,ih*{w}/{h}):min(ih,iw*{h}/{w})'"))
                                .arg(&output_file)
                                .status()
                                .await
                        });
                    }
                    while let Some(task) = set.join_next().await {
                        let status = task??;
                        if let Some(code) = status.code()
                            && code != 0
                        {
                            return Err(DownloadError::ImageMagick(code));
                        }
                    }
                }
                _ => {
                    println!("Copying file {}", target_file.display());
                    copy(cache_file, target_file).await?;
                }
            }
        }
        Ok(())
    }
}

#[derive(strum::FromRepr, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(not(feature = "client-side"), allow(unused))]
#[repr(i8)]
pub(crate) enum YouTubePlayerState {
    #[default]
    Unstarted = -1,
    Ended = 0,
    Playing = 1,
    Paused = 2,
    Buffering = 3,
    VideoQueued = 5,
}

#[component]
pub(crate) fn YouTube(
    #[prop(into)] video: YoutubeVideo,
    #[prop(optional)] max_width: Option<&'static str>,
    #[prop(optional)] max_height: Option<&'static str>,
    #[prop(optional)] player_state: Option<WriteSignal<YouTubePlayerState>>,
    #[prop(optional)] comment: Option<&'static str>,
) -> impl IntoView {
    #[cfg(feature = "ssr")]
    {
        let context = Owner::current().unwrap().shared_context().unwrap();
        let video = video.clone();
        let future = async move {
            downloader::download_youtube_thumbnail(&video)
                .await
                .unwrap()
        }
        .into_future();
        context.defer_stream(Box::pin(future));
    }
    let thumbnail_attrs = {
        use std::sync::{
            Arc,
            atomic::{AtomicUsize, Ordering},
        };
        #[derive(Clone)]
        struct ActiveYoutubeTags(Arc<AtomicUsize>);
        let current_active = once_by_type(
            true,
            || (ActiveYoutubeTags(Arc::new(0.into())), None),
            |ActiveYoutubeTags(count)| count.clone(),
        );
        let lazy_attr = (current_active.fetch_add(1, Ordering::Relaxed) > 0).then_some("lazy");

        Owner::on_cleanup(move || {
            current_active.fetch_sub(1, Ordering::Relaxed);
        });

        (attr::Attr(attr::Loading, lazy_attr),)
    };

    request_third_party_cookies();
    let consent_mode =
        get_local_storage_value::<YoutubeConsentType>().unwrap_or_else(|_| Signal::from(None));
    let ratio = Oco::<str>::Counted(format!("{}/{}", video.width, video.height).into());
    let href: Oco<str> = Oco::Counted(format!("https://youtube.com/watch?v={}", video.id).into());
    let youtube_id: (&str, Oco<str>) = (
        "--youtube-id",
        Oco::Counted(format!("\"{}\"", video.id).into()),
    );
    move || {
        let do_show = show_youtube_consent_dialog();
        let youtube_id = youtube_id.clone();
        let comment = move || comment.map(|x| html::p().class(("comment", true)).child(x));
        let (currently_playing, set_currently_playing) = signal(false);
        let (iframe_loading, set_iframe_loading) = signal(true);
        let regular_link = || {
            let author_url = video
                .author_url
                .clone()
                .filter(|x| x.starts_with("http://") || x.starts_with("https://"));
            let show_consent = move |e: ev::MouseEvent| {
                if consent_mode.get().is_none() {
                    do_show(e)
                }
            };
            let author = video.author_name.clone().map(|author_name| {
                view! {
                    <a class:author=true class:no-shinies=true href=author_url>
                        {author_name}
                    </a>
                }
            });
            let href = href.clone();
            let thumbnail = cfg_select! {
                feature = "ffmpeg" => {
                    view! {
                    <picture class:thumbnail=true>
                        <source type="image/avif" srcset=format!("/youtube/{}.avif", video.id) />
                        <source type="image/webp" srcset=format!("/youtube/{}.webp", video.id) />
                        <img
                            alt
                            src=format!("/youtube/{}.jpg", video.id)
                            {..thumbnail_attrs.clone()}
                        />
                    </picture>
                    }
                }
                _ => {
                    view! {
                    <img
                        alt
                        class:thumbnail=true
                        src=format!("/youtube/{}.jpg", video.id)
                        {..thumbnail_attrs.clone()}
                    />
                    }
                }
            };
            view! {
                <div class:simple-embed=true>
                    <span class:meta=true>
                        <a
                            href=href.clone()
                            class:no-shinies=true
                            class:title=true
                            on:click=show_consent.clone()
                        >
                            {video.title.clone()}
                        </a>
                        {author}
                    </span>
                    <a
                        class:logo=true
                        class:no-shinies=true
                        href=href
                        aria-label="Play on YouTube"
                        title="YouTube"
                        on:click=show_consent
                    ></a>
                    {thumbnail}
                    {comment}
                </div>
            }
        };
        let embed_src = match consent_mode.get() {
            Some(YoutubeConsentType::PlainLink) | None => Either::Right(regular_link()),
            Some(YoutubeConsentType::NoCookieDomain) => Either::Left("-nocookie"),
            Some(YoutubeConsentType::RegularYoutube) => Either::Left(""),
        };
        let ratio = ratio.clone();
        let embedded = embed_src.map_left(move |url_suffix| {
            let (url, set_url) = signal(None);
            Effect::new(move || {
                set_url.set(Some(format!(
                    "https://www.youtube{url_suffix}.com/embed/{}?enablejsapi=1",
                    video.id
                )));
                set_iframe_loading.set(true);
            });
            let iframe = NodeRef::<html::Iframe>::new();
            #[allow(clippy::let_unit_value)]
            let player_state_attrs = {
                cfg_select! {
                    feature = "client-side" => {
                        use crate::helpers::unique_index;
                        use js_sys::{Function, JSON, Object, Reflect};
                        use leptos::ev::EventDescriptor;
                        use wasm_bindgen::{closure::Closure, convert::TryFromJsValue, prelude::*};
                        use web_sys::{EventListener, console};
                        let my_index = unique_index();

                        #[derive(Clone)]
                        struct AnyYoutubeCurrentlyPlaying(RwSignal<Option<usize>>);
                        let currently_playing = once_by_type(
                            true,
                            move || (AnyYoutubeCurrentlyPlaying(RwSignal::new(None)), None),
                            |AnyYoutubeCurrentlyPlaying(signal)| signal,
                        );

                        let my_state = RwSignal::new(YouTubePlayerState::default());
                        Effect::new(move || {
                            let new_state = my_state.get();
                            set_currently_playing.set(matches!(
                                new_state,
                                YouTubePlayerState::Playing | YouTubePlayerState::Buffering
                            ));
                            if let Some(player_state) = player_state {
                                let mut writer = player_state.write();
                                if *writer == new_state {
                                    writer.untrack()
                                } else {
                                    *writer = new_state;
                                }
                            }
                            let mut current = currently_playing.write();
                            match (*current, new_state) {
                                (Some(index), YouTubePlayerState::Playing) if index != my_index => {
                                    *current = Some(my_index);
                                }
                                (None, YouTubePlayerState::Playing) => {
                                    *current = Some(my_index);
                                }
                                (Some(index), _) if index == my_index => {
                                    *current = None;
                                }
                                _ => {
                                    current.untrack();
                                }
                            }
                        });

                        let listener = EventListener::new();
                        fn callback(
                            iframe: NodeRef<html::Iframe>,
                            player_state: RwSignal<YouTubePlayerState>,
                        ) -> impl FnMut(<ev::message as EventDescriptor>::EventType)
                        {
                            move |msg| {
                                let Some(content_window) =
                                    iframe.get_untracked().and_then(|x| x.content_window())
                                else {
                                    return;
                                };
                                if msg.source().as_ref() != Some(content_window.as_ref()) {
                                    return;
                                }
                                if let Some(data) = msg.data().as_string() {
                                    let Ok(data) = JSON::parse(&data)
                                        .and_then(Object::<JsValue>::try_from_js_value)
                                        .map_err(|e| {
                                            console::error_2(
                                                &JsValue::from_str("Could not parse JSON"),
                                                &e,
                                            )
                                        })
                                    else {
                                        return;
                                    };
                                    let Ok(event) =
                                        Reflect::get_str(&data, &JsString::from(intern("event")))
                                            .map_err(|e| {
                                                console::error_2(
                                                    &JsValue::from_str("Invalid event"),
                                                    &e,
                                                )
                                            })
                                    else {
                                        return;
                                    };
                                    if event == Some(JsString::from(intern("infoDelivery")).into())
                                        && let Ok(Some(info)) =
                                            Reflect::get_str(&data, &JsString::from(intern("info")))
                                                .map(|x| x.map(|x| x.into()))
                                        && let Ok(Some(new_player_state)) = Reflect::get_str(
                                            &info,
                                            &JsString::from(intern("playerState")),
                                        )
                                        && let Some(state_f64) = new_player_state.as_f64()
                                        && let Some(state) =
                                            YouTubePlayerState::from_repr(state_f64 as i8)
                                    {
                                        let mut writer = player_state.write();
                                        if *writer == state {
                                            writer.untrack();
                                        } else {
                                            *writer = state;
                                        }
                                    }
                                }
                            }
                        }
                        let callback = Closure::new(callback(iframe, my_state));
                        let callback =
                            Function::try_from_js_value(callback.into_js_value()).unwrap();
                        listener.set_handle_event(&callback);
                        let event = ev::message.name();
                        if let Err(e) =
                            window().add_event_listener_with_event_listener(&event, &listener)
                        {
                            console::error_2(&JsString::from("Failed to add event listener"), &e);
                        }
                        Owner::on_cleanup(move || {
                            if let Err(e) = window()
                                .remove_event_listener_with_event_listener(&event, &listener)
                            {
                                console::error_2(
                                    &JsString::from("Failed to remove event listener"),
                                    &e,
                                );
                            }
                        });
                        Effect::new(move || {
                            let Some(current) = currently_playing.get() else {
                                return;
                            };
                            if current == my_index {
                                return;
                            };
                            if matches!(
                                my_state.get_untracked(),
                                YouTubePlayerState::Playing | YouTubePlayerState::Buffering
                            ) {
                                let Some(window) =
                                    iframe.get_untracked().and_then(|x| x.content_window())
                                else {
                                    return;
                                };
                                let object = JsString::from(intern(
                                    r#"{"event": "command", "func": "pauseVideo"}"#,
                                ));
                                if let Err(e) = window.post_message(&object, "*") {
                                    console::error_1(&e);
                                }
                            }
                        });
                        (
                            ev::on(ev::DOMContentLoaded, move |_| {
                                set_iframe_loading.set(false);
                            }),
                            ev::on(ev::load, move |_| {
                                let Some(window) =
                                    iframe.get_untracked().and_then(|x| x.content_window())
                                else {
                                    return;
                                };
                                let object = JsString::from(intern(r#"{"event": "listening"}"#));
                                if let Err(e) = window.post_message(&object, "*") {
                                    console::error_1(&e);
                                }
                                set_iframe_loading.set(false);
                            }),
                            ev::on(ev::error, move |_| {
                                set_iframe_loading.set(false);
                            }),
                        )
                    }
                    _ => {
                        _ = (player_state, set_currently_playing);
                    }
                }
            };
            view! {
                <iframe
                    {..player_state_attrs}
                    node_ref=iframe
                    class:youtube-embed=true
                    src=url
                    allow="fullscreen; encrypted-media; picture-in-picture"
                    referrerpolicy="origin"
                    {..custom_attribute("frameBorder", 0)}
                />
                {comment}
            }
            .into_inner()
        });
        view! {
            <div
                class:youtube-embed=true
                class:playing=currently_playing
                class:loading=iframe_loading
                style=("--aspect-ratio", ratio.clone())
                style=youtube_id.clone()
                style:max-width=max_width
                style:max-height=max_height
            >
                {embedded}
            </div>
        }
    }
}

#[derive(Clone)]
struct YoutubeDialog(NodeRef<html::Dialog>);

impl YoutubeDialog {
    fn singleton() -> NodeRef<html::Dialog> {
        once_by_type(true, || (Self(NodeRef::new()), None), |Self(node)| node)
    }
}

fn show_youtube_consent_dialog() -> impl Clone + Fn(ev::MouseEvent) {
    let node = YoutubeDialog::singleton();
    register_interested_owner::<YoutubeConsentType>();
    let saved_value = get_local_storage_value::<YoutubeConsentType>().unwrap_or(Signal::from(None));
    move |event| {
        if let Some(node) = node.get_untracked()
            && node.show_modal().is_ok()
            && saved_value.get_untracked().is_none()
        {
            event.prevent_default();
            event.stop_propagation();
        }
    }
}

#[component]
pub(crate) fn ThirdPartyConsentDialogs() -> impl IntoView {
    let node = YoutubeDialog::singleton();
    let wants_youtube = {
        let has = has_interested_owners::<YoutubeConsentType>();
        move || has.get()
    };
    let saved_value = get_local_storage_value::<YoutubeConsentType>().unwrap_or(Signal::from(None));
    Effect::new(move |prev| {
        let show = saved_value.get().is_none();
        if prev != Some(show)
            && let Some(node) = node.get()
        {
            node.close();
        }
        show
    });
    view! {
        <Show when=wants_youtube>
            <dialog node_ref=node closedby="any" class:cookie-consent=true>
                <form method="dialog">
                    <input type="submit" value="\u{2bbe}" />
                </form>
                <h1>Settings for <span class:with-youtube-logo=true>YouTube</span></h1>
                <span>"How do you want to interact with YouTube on this web page?"</span>
                <YoutubeConsent />
            </dialog>
        </Show>
    }
}
