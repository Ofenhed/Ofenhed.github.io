use std::borrow::Cow;

use strum::{AsRefStr, EnumString, IntoStaticStr, VariantArray};

use leptos::{attr::custom::custom_attribute, either::Either, prelude::*};

use crate::{
    cookie_consent::request_third_party_cookies, helpers::ImgDef,
    local_storage::get_local_storage_value,
};

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
    ($id:literal) => {{
        let oembed::OembedData {
            title,
            author_url,
            author_name,
            #[cfg(feature = "ssr")]
            thumbnail_url,
            content,
            ..
        } = oembed::oembed! {
                "https://www.youtube.com/oembed",
                "https://www.youtube.com/watch?v=" + $id
        };
        let (width, height) = match content {
            oembed::OembedType::Video { width, height, .. } => (width, height),
        };
        $crate::third_party::YoutubeVideo {
            id: $id,
            title,
            author_url,
            author_name,
            #[cfg(feature = "ssr")]
            thumbnail_url,
            width,
            height,
        }
    }};
    ($id:literal ($width:literal : $height:literal)) => {{
        let mut yv = youtube!($id);
        (yv.width, yv.height) = ($width, $height);
        yv
    }};
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
    use std::{borrow::Cow, path::PathBuf};
    use tokio::{fs::*, io::AsyncWriteExt};
    #[derive(thiserror::Error, Debug)]
    pub enum DownloadError {
        #[error(transparent)]
        Reqwest(#[from] reqwest::Error),
        #[error("Invalid HTTP response: {0}")]
        InvalidHttp(StatusCode),
        #[error(transparent)]
        Io(#[from] std::io::Error),
    }
    pub async fn download_youtube_thumbnail(video: &YoutubeVideo) -> Result<(), DownloadError> {
        let options = use_context::<LeptosOptions>()
            .expect("YouTube integration requires LeptosOptions as context");
        let target_root = PathBuf::from(format!("{}/youtube", options.site_root));
        create_dir_all(&target_root).await?;
        let mut target_file = target_root;
        target_file.push(format!("{}.jpg", video.id));
        let Ok(mut image_file) = File::create_new(&target_file).await else {
            return Ok(());
        };
        let client = reqwest::Client::new();

        let mut image = client
            .get(&*video.thumbnail_url.clone().unwrap_or_else(|| Cow::Owned(format!("https://www.youtube.com/oembed?url=https://www.youtube.com/watch?v={}&format=json", video.id))))
            .send()
            .await?;
        if !image.status().is_success() {
            return Err(DownloadError::InvalidHttp(image.status()));
        }
        while let Some(chunk) = image.chunk().await? {
            image_file.write_all(&chunk).await?;
        }
        Ok(())
    }
}

#[component]
pub fn YouTube(
    #[prop(into)] video: YoutubeVideo,
    #[prop(optional)] max_width: Option<&'static str>,
    #[prop(optional)] max_height: Option<&'static str>,
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

    request_third_party_cookies();
    let consent_mode = match get_local_storage_value::<YoutubeConsentType>() {
        Ok(value) => value,
        Err(_) => Signal::derive(|| None),
    };
    let consent_mode = move || consent_mode.get().unwrap_or_default();
    let ratio = Oco::<str>::Counted(format!("{}/{}", video.width, video.height).into());
    move || {
        let regular_link = {
            let ratio = ratio.clone();
            let author_url = video
                .author_url
                .clone()
                .filter(|x| x.starts_with("http://") || x.starts_with("https://"));
            let author = video.author_name.clone().map(|author_name| {
                view! {
                    <a class:author href=author_url>
                        {author_name}
                    </a>
                }
            });
            let href: Oco<str> =
                Oco::Counted(format!("https://youtube.com/watch?v={}", video.id).into());
            view! {
                <div
                    class:simple-embed
                    class:youtube-embed
                    style:aspect-ratio=ratio.clone()
                    style:max-width=max_width
                    style:max-height=max_height
                >
                    <span class:meta>
                        <a href=href.clone() class:no-shinies class:title>
                            {video.title.clone()}
                        </a>
                        {author}
                    </span>
                    <a class:logo class:no-shinies href=href title="YouTube"></a>
                    <img
                        alt
                        class:thumbnail
                        src=format!("/youtube/{}.jpg", video.id)
                        {..ImgDef()}
                    />
                </div>
            }
        };
        let embed_src = match consent_mode() {
            YoutubeConsentType::PlainLink => Either::Right(regular_link),
            YoutubeConsentType::NoCookieDomain => Either::Left("-nocookie"),
            YoutubeConsentType::RegularYoutube => Either::Left(""),
        };
        let ratio = ratio.clone();
        embed_src.map_left(move |url_suffix| {
            view! {
                <iframe
                    class:youtube-embed
                    style:aspect-ratio=ratio.clone()
                    style:max-width=max_width
                    style:max-height=max_height
                    src=format!("https://www.youtube{url_suffix}.com/embed/{}", video.id)
                    allow="fullscreen; encrypted-media; picture-in-picture"
                    referrerpolicy="origin"
                    {..custom_attribute("frameBorder", 0)}
                />
            }
            .into_inner()
        })
    }
}
