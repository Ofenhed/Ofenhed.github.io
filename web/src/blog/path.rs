use super::{CurrentPage, SortBy, SortInvert, TagFilter};
use crate::helpers::{into_static_str, split_prefix, str_offset_unchecked};

use super::BlogEntryMeta;
use leptos::oco::Oco;
use leptos_router::{PartialPathMatch, PathSegment, PossibleRouteMatch};
use std::{borrow::Cow, str::FromStr};

pub(crate) fn maybe_path_match<'a>(
    remaining: &'a str,
    segments: Vec<(Cow<'static, str>, String)>,
    matched: &'a str,
) -> Option<PartialPathMatch<'a>> {
    if matches!(remaining.chars().next(), None | Some('/')) {
        Some(PartialPathMatch::new(remaining, segments, matched))
    } else {
        None
    }
}

pub(crate) fn format_path(p: impl IntoIterator<Item = PathSegment>) -> Oco<'static, str> {
    Oco::Owned(
        p.into_iter()
            .map(|x| format!("/{}", x.as_raw_str()))
            .collect::<String>(),
    )
}

impl PossibleRouteMatch for BlogEntryMeta {
    fn optional(&self) -> bool {
        false
    }
    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let mut matched_len = 0;
        let mut param_offset = 0;
        let mut param_len = 0;
        let mut test = path.chars();

        if let Some('/') = test.next() {
            matched_len += 1;
            param_offset = 1;
        }

        for char in test {
            if char.is_numeric() {
                matched_len += char.len_utf8();
                param_len += char.len_utf8();
            } else {
                break;
            }
        }

        if matched_len == param_offset {
            return None;
        }

        let matched_num = &path[param_offset..param_len + param_offset];
        match u32::from_str(matched_num) {
            Ok(id) if id == self.uid => {
                if let (Some(locale), true) = (self.locale, self.path_locale) {
                    locale
                        .test(&path[param_offset + param_len..])
                        .map(|m| {
                            let remaining_offset =
                                unsafe { str_offset_unchecked(path, m.remaining()) }.unwrap();
                            let (matched_offset, matched_len) = {
                                let matched = m.matched();
                                (
                                    unsafe { str_offset_unchecked(path, matched) }.unwrap(),
                                    matched.len(),
                                )
                            };
                            PartialPathMatch::new(
                                &path[remaining_offset..],
                                vec![],
                                &path[param_offset..matched_offset + matched_len],
                            )
                        })
                        .or(None)
                } else {
                    Some(PartialPathMatch::new(
                        &path[param_offset + param_len..],
                        vec![],
                        &path[param_offset..param_offset + param_len],
                    ))
                }
            }
            _ => None,
        }
    }
    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        path.push(PathSegment::Static(Cow::Owned(self.uid.to_string())));
        if let (Some(locale), true) = (self.locale, self.path_locale) {
            locale.generate_path(path)
        }
    }
}

impl PossibleRouteMatch for SortBy {
    fn optional(&self) -> bool {
        false
    }

    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let path = path.strip_prefix('/').unwrap_or(path);

        if let SortBy::Default = self {
            Some(PartialPathMatch::new(path, vec![], path))
        } else if let Some(sort_argument) = path.strip_prefix("sort/") {
            if let Some(after) = sort_argument.strip_prefix(into_static_str(self)) {
                let after_offset = unsafe { str_offset_unchecked(path, after) }.unwrap();
                maybe_path_match(after, vec![], &path[..after_offset])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        match self {
            SortBy::Default => (),
            x => {
                path.push(PathSegment::Static(Cow::Borrowed("sort")));
                path.push(PathSegment::Static(Cow::Borrowed(into_static_str(x))));
            }
        }
    }
}

impl PossibleRouteMatch for SortInvert {
    fn optional(&self) -> bool {
        false
    }

    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let path = path.strip_prefix('/').unwrap_or(path);
        let SortInvert(do_sort) = self;

        if *do_sort {
            if let Some((p, after)) = split_prefix(path, "invert") {
                maybe_path_match(after, vec![], p)
            } else {
                None
            }
        } else {
            Some(PartialPathMatch::new(path, vec![], &path[0..0]))
        }
    }

    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        if self.0 {
            path.push(PathSegment::Static(Cow::Borrowed("invert")));
        }
    }
}

impl PossibleRouteMatch for TagFilter {
    fn optional(&self) -> bool {
        false
    }

    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let path = path.strip_prefix('/').unwrap_or(path);
        if let Some(tag) = path.strip_prefix("tag/") {
            if let Some(remaining) = tag.strip_prefix(into_static_str(self.0)) {
                let path_offset = unsafe { str_offset_unchecked(path, remaining) }.unwrap();
                maybe_path_match(remaining, vec![], &path[..path_offset])
            } else {
                None
            }
        } else {
            None
        }
    }

    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        path.push(PathSegment::Static(Cow::Borrowed("tag")));
        path.push(PathSegment::Static(Cow::Borrowed(into_static_str(self.0))));
    }
}

impl PossibleRouteMatch for CurrentPage {
    fn optional(&self) -> bool {
        false
    }

    fn test<'a>(&self, path: &'a str) -> Option<PartialPathMatch<'a>> {
        let path = path.strip_prefix('/').unwrap_or(path);
        assert_eq!("test".strip_prefix("test"), Some(""));
        if self.0 == 0 {
            Some(PartialPathMatch::new(path, vec![], &path[0..0]))
        } else {
            if let Some(page) = path.strip_prefix("page/") {
                if let Some(remaining) = page.strip_prefix(&format!("{}", self.0 + 1)) {
                    let path_offset = unsafe { str_offset_unchecked(path, remaining) }.unwrap();
                    maybe_path_match(remaining, vec![], &path[..path_offset])
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    fn generate_path(&self, path: &mut Vec<PathSegment>) {
        if self.0 > 0 {
            path.push(PathSegment::Static(Cow::Borrowed("page")));
            path.push(PathSegment::Static(Cow::Owned(format!("{}", self.0 + 1))));
        }
    }
}
