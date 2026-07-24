use std::borrow::Cow;

#[cfg_attr(feature = "clonable", derive(Clone))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "test", derive(PartialEq, Eq))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case", tag = "type"))]
pub enum OembedType<'a> {
    Video {
        #[cfg_attr(feature = "serde", serde(borrow))]
        html: Cow<'a, str>,
        width: usize,
        height: usize,
    },
}

#[cfg_attr(feature = "clonable", derive(Clone))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "test", derive(PartialEq, Eq))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum OembedVersion {
    #[cfg_attr(feature = "serde", serde(rename = "1.0"))]
    Ver1_0,
}

#[cfg_attr(feature = "clonable", derive(Clone))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "test", derive(PartialEq, Eq))]
#[cfg_attr(debug_assertions, derive(Debug))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub struct OembedData<'a> {
    pub version: OembedVersion,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub title: Option<Cow<'a, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub author_name: Option<Cow<'a, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub author_url: Option<Cow<'a, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub provider_name: Option<Cow<'a, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub provider_url: Option<Cow<'a, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub cache_age: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub thumbnail_url: Option<Cow<'a, str>>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub thumbnail_width: Option<usize>,
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub thumbnail_height: Option<usize>,
    #[cfg_attr(feature = "serde", serde(borrow), serde(flatten))]
    pub content: OembedType<'a>,
}
