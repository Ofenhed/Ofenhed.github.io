use std::{cell::LazyCell, sync::atomic::AtomicUsize};

use leptos::{
    attr::{Attr, Loading, custom::custom_attribute},
    ev, logging,
    prelude::*,
    tachys::view::iterators::StaticVec,
};
use leptos_router::{
    MatchNestedRoutes, any_nested_route::IntoAnyNestedRoute as _, hooks::use_location,
};

/// Zero Width Non-Joiner
///
/// Can be used to block phone number detection for browsers that ignore the meta tag
pub(crate) const ZWNJ: char = '\u{200C}';

#[cfg_attr(feature = "ssr", allow(unused))]
pub(crate) trait ScopedTimeout {
    fn set_scoped_timeout(&self, timeout: std::time::Duration, action: impl 'static + FnOnce());
    fn request_scoped_animation_frame(&self, action: impl 'static + FnOnce());
}

impl ScopedTimeout for Owner {
    fn set_scoped_timeout(&self, timeout: std::time::Duration, action: impl 'static + FnOnce()) {
        let owner = self.downgrade();
        set_timeout(
            move || {
                if let Some(owner) = owner.upgrade() {
                    owner.with(action);
                }
            },
            timeout,
        );
    }
    fn request_scoped_animation_frame(&self, action: impl 'static + FnOnce()) {
        let owner = self.downgrade();
        request_animation_frame(move || {
            if let Some(owner) = owner.upgrade() {
                owner.with(action);
            }
        });
    }
}

#[cfg_attr(feature = "ssr", allow(unused))]
pub(crate) fn set_scoped_timeout(timeout: std::time::Duration, action: impl 'static + FnOnce()) {
    let Some(owner) = Owner::current() else {
        return;
    };
    owner.set_scoped_timeout(timeout, action)
}

#[cfg_attr(feature = "ssr", allow(unused))]
pub(crate) fn request_scoped_animation_frame(action: impl 'static + FnOnce()) {
    let Some(owner) = Owner::current() else {
        return;
    };
    owner.request_scoped_animation_frame(action)
}

#[cfg_attr(feature = "ssr", allow(unused))]
pub(crate) struct IntervalIterator<I, F> {
    it: I,
    interval: std::time::Duration,
    action: F,
    owner: WeakOwner,
}

#[cfg_attr(feature = "ssr", allow(unused))]
impl<I: 'static + Iterator, F: 'static + Fn(<I as Iterator>::Item)> IntervalIterator<I, F> {
    pub(crate) fn into_scoped_timeout(mut self) {
        let Some(i) = self.it.next() else { return };
        let Some(owner) = self.owner.upgrade() else {
            logging::warn!("Lost iterator owner");
            return;
        };
        let interval = self.interval;
        owner.set_scoped_timeout(interval, move || {
            (self.action)(i);
            self.into_scoped_timeout();
        });
    }
}

pub fn into_static_str<T>(source: T) -> &'static str
where
    T: Into<&'static str>,
{
    source.into()
}

#[cfg_attr(feature = "ssr", allow(unused))]
pub(crate) trait IntoIntervalIterator<F>
where
    Self: Iterator + Sized,
{
    #[must_use]
    fn on_interval(self, interval: std::time::Duration, action: F) -> IntervalIterator<Self, F>;
}

#[cfg_attr(feature = "ssr", allow(unused))]
impl<I: Iterator, F: 'static + Fn(<Self as Iterator>::Item)> IntoIntervalIterator<F> for I {
    fn on_interval(self, interval: std::time::Duration, action: F) -> IntervalIterator<Self, F> {
        let Some(owner) = Owner::current() else {
            panic!("on_interval called without an owner");
        };
        IntervalIterator {
            it: self,
            interval,
            action,
            owner: owner.downgrade(),
        }
    }
}

#[component(transparent)]
pub(crate) fn ForRoute<X, R: Clone + Send + 'static + MatchNestedRoutes, F: Fn(X) -> R>(
    each: impl IntoIterator<Item = X>,
    children: F,
) -> impl MatchNestedRoutes + Clone {
    let entries = StaticVec::from(each.into_iter().map(children).collect::<Vec<_>>());
    entries.into_any_nested_route()
}

pub(crate) fn once_by_type<T: Clone + Send + Sync + 'static, R>(
    in_root: bool,
    x: impl Fn() -> (T, R),
    f: impl Fn(T) -> R,
) -> R {
    let root_context = LazyCell::new(|| {
        let mut owner = Owner::current().unwrap();
        while let Some(parent) = owner.parent() {
            owner = parent;
        }
        owner
    });
    if let Some(context) = use_context().or_else(|| root_context.use_context_bidirectional()) {
        f(context)
    } else {
        let make_context = move || {
            let (context, r) = x();
            provide_context(context);
            r
        };
        if in_root {
            root_context.with(make_context)
        } else {
            make_context()
        }
    }
}

#[component]
pub(crate) fn NoWasm(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    #[derive(Clone, Copy)]
    struct NoWasmScriptLoaded;
    let init_script = once_by_type(
        false,
        || {
            #[cfg(not(feature = "ssr"))]
            let script = "";
            #[cfg(feature = "ssr")]
            let script = js_macro::minify_js! {
                addEventListener("DOMContentLoaded", (event) => {
                    const has_wasm = (() => {
                        try {
                            if (typeof WebAssembly === "object"
                                && typeof WebAssembly.instantiate === "function") {
                                const module = new WebAssembly.Module(Uint8Array.of(0x0, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00));
                                if (module instanceof WebAssembly.Module)
                                    return new WebAssembly.Instance(module) instanceof WebAssembly.Instance;
                            }
                        } catch (e) {
                        }
                        return false;
                    })();
                    if (!has_wasm) {
                        document.querySelectorAll("template.wasm-fallback").forEach((template) => {
                            template.parentElement.replaceChild(document.importNode(template.content, true), template);
                        });
                    }
                });
            };
            (
                NoWasmScriptLoaded,
                Some(view! { <script nonce=use_nonce()>{script}</script> }),
            )
        },
        |_| None,
    );

    view! {
        {init_script}
        <template class:wasm-fallback=true {..custom_attribute("shadowrootclonable", ())}>
            {children.map(|x| x())}
        </template>
    }
}

type ImgDefAttr = (Attr<Loading, &'static str>,);

#[component(transparent)]
pub(crate) fn ImgDef() -> ImgDefAttr {
    (Attr(Loading, "lazy"),)
}

#[derive(Clone)]
struct FootnoteInner {
    uid: usize,
    name: Signal<Oco<'static, str>>,
    children: ChildrenFn,
}
type FootnotesInner = (
    RwSignal<Option<Oco<'static, str>>>,
    RwSignal<Vec<FootnoteInner>>,
);
fn footnotes() -> FootnotesInner {
    #[cfg_attr(debug_assertions, derive(Debug))]
    #[derive(Clone)]
    struct FootnotesHolder(FootnotesInner);

    once_by_type(
        true,
        || {
            let (active, visible) = (RwSignal::new(None), RwSignal::new(vec![]));
            (FootnotesHolder((active, visible)), (active, visible))
        },
        |FootnotesHolder((active, visible))| (active, visible),
    )
}

type AbbrList = Vec<(usize, Signal<String>)>;

fn abbrs() -> (ReadSignal<AbbrList>, WriteSignal<AbbrList>) {
    #[cfg_attr(debug_assertions, derive(Debug))]
    #[derive(Clone)]
    struct AllAbbrs<T>(ReadSignal<T>, WriteSignal<T>);
    once_by_type(
        true,
        || {
            let (reader, writer) = signal::<Vec<(usize, Signal<String>)>>(vec![]);
            (AllAbbrs(reader, writer), (reader, writer))
        },
        |AllAbbrs(reader, writer)| (reader, writer),
    )
}

macro_rules! scroll_into_view {
    ($element:ident) => {
        #[cfg(feature = "client-side")]
        {
            $element.scroll_into_view_with_scroll_into_view_options(&{
                let opts = web_sys::ScrollIntoViewOptions::new();
                opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                opts.set_block(web_sys::ScrollLogicalPosition::Center);
                opts
            })
        }
        #[cfg(not(feature = "client-side"))]
        {
            _ = $element;
        }
    };
}

pub(crate) fn footnote_ref(target: &str) -> Oco<'static, str> {
    Oco::Owned(format!("{target}-source"))
}

pub(crate) fn reset_footnote() {
    let (active, _) = footnotes();
    active.set(None);
}

#[component]
pub(crate) fn Footnotes() -> impl IntoView {
    let (active, footnotes) = footnotes();
    let current_hash = use_location().hash;
    let is_current = move |name: Signal<Oco<'static, str>>| {
        move || {
            #[cfg(feature = "client-side")]
            {
                name.with(|name| {
                    active.with(|x| x.as_ref().map(|x| *x == *name).unwrap_or(false))
                        || current_hash.with(|hash| hash.strip_prefix('#').unwrap_or(hash) == *name)
                })
            }
            #[cfg(not(feature = "client-side"))]
            {
                let _ = (current_hash, name.clone());
                false
            }
        }
    };
    let on_click = move |source_id: Oco<'static, str>| {
        move |e: ev::MouseEvent| {
            if let Some(footnote) = document().get_element_by_id(&source_id) {
                e.prevent_default();
                scroll_into_view!(footnote);
                active.set(Some(source_id.clone()));
            }
        }
    };

    let return_link = move |target_id: Oco<'static, str>| {
        view! {
            <a
                on:click=on_click(target_id.clone())
                class="footnote-return-link"
                aria-label="Back to content"
                href=format!("#{target_id}")
            />
        }
        .into_inner()
    };
    view! {
        <Show when=move || footnotes.with(|x| !x.is_empty())>
            <footer>
                <For
                    each=move || {
                        footnotes
                            .with(|x| {
                                x.iter()
                                    .map(|FootnoteInner { name, children, .. }| (
                                        *name,
                                        (*children)(),
                                    ))
                                    .collect::<Vec<_>>()
                            })
                    }
                    key=|(name, _)| name.get()
                    let((name, inner))
                >
                    <div id=name aria-current=is_current(name)>
                        <div>{inner}</div>
                        {return_link(Oco::Owned(format!("{}-source", name.get())))}
                    </div>
                </For>
            </footer>
        </Show>
    }
}

#[component]
pub(crate) fn Footnote(
    #[prop(into, optional)] id: Option<Oco<'static, str>>,
    children: ChildrenFn,
) -> impl IntoView {
    let (active, footnotes) = footnotes();
    static FOOT_IDX: AtomicUsize = AtomicUsize::new(1);
    let uid = FOOT_IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let footnote_name = {
        let id = id.clone();
        move || {
            let my_uid = uid;
            id.clone().unwrap_or_else(move || {
                let number = 1 + footnotes.with(|x| {
                    if let Some(item) = x.iter().find(|FootnoteInner { uid, .. }| *uid == my_uid) {
                        x.element_offset(item).unwrap()
                    } else {
                        x.len()
                    }
                });
                Oco::Counted(format!("footnote-{number}").into())
            })
        }
    };

    let my_footnote = FootnoteInner {
        uid,
        name: Signal::derive(footnote_name.clone()),
        children,
    };

    let current_hash = use_location().hash;
    let is_current = move |name: Oco<'static, str>| {
        move || {
            #[cfg(feature = "client-side")]
            {
                active.with(|x| x.as_ref().map(|x| *x == *name).unwrap_or(false))
                    || current_hash.with(|hash| hash.strip_prefix('#').unwrap_or(hash) == name)
            }
            #[cfg(not(feature = "client-side"))]
            {
                let _ = (current_hash, name.clone());
                false
            }
        }
    };

    Owner::on_cleanup({
        move || {
            footnotes.update(move |x| {
                if let Some(elem) = x
                    .iter()
                    .find(move |FootnoteInner { uid: elem_uid, .. }| uid == *elem_uid)
                {
                    x.remove(x.element_offset(elem).unwrap());
                }
            });
        }
    });

    footnotes.update(move |x| {
        x.push(my_footnote);
    });

    let on_click = {
        let name = footnote_name.clone();
        move |e: ev::MouseEvent| {
            let name = name();
            if let Some(footnote) = document().get_element_by_id(&name) {
                active.set(Some(name));
                scroll_into_view!(footnote);
                e.prevent_default();
            }
        }
    };

    let footnote_source = footnote_ref(&footnote_name());
    view! {
        <a
            on:click=on_click
            id=footnote_source.clone()
            aria-describedby=footnote_name.clone()
            aria-current=move || is_current(footnote_source.clone())
            class="footnote-link"
            href=move || format!("#{}", footnote_name())
        />
    }
}

#[component]
pub(crate) fn Url(children: TypedChildrenFn<&'static str>) -> impl IntoView {
    let url = (children.into_inner())().into_inner();
    view! {
        <a class="just-url" href=url>
            {url}
        </a>
    }
}

#[component]
pub(crate) fn Abbr<T: IntoView + 'static>(
    #[prop(into)] title: Signal<String>,
    #[prop(into, optional)] suffix: Signal<Option<String>>,
    children: TypedChildrenMut<T>,
) -> impl IntoView {
    let (read_abbrs, write_abbrs) = abbrs();
    static FOOT_IDX: AtomicUsize = AtomicUsize::new(1);
    let uid = FOOT_IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    write_abbrs.update(|abbrs| abbrs.push((uid, title)));

    Owner::on_cleanup(move || {
        write_abbrs.update(move |abbrs| {
            if let Some((index, _)) = abbrs.iter().enumerate().find(|(_, (id, _))| *id == uid) {
                abbrs.remove(index);
            }
        });
    });

    let first_of_abbr = move || {
        title.with(|abbr| {
            read_abbrs.with(move |abbrs| {
                abbrs
                    .iter()
                    .find(|(_, name)| name.with(|name| name == abbr))
                    .map(|x| x.0)
                    == Some(uid)
            })
        })
    };
    view! {
        <abbr
            title=move || {
                suffix
                    .with(|s| {
                        if let Some(s) = s {
                            title.with(|title| format!("{title}{s}"))
                        } else {
                            title.get()
                        }
                    })
            }
            class:first-of-abbr=first_of_abbr
        >
            {children.into_inner()}
            {suffix}
        </abbr>
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub(crate) enum StrOffsetError {
    #[error("Not contained")]
    NotContained,
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
}

pub(crate) unsafe fn str_offset_unchecked(
    haystack: &str,
    needle: &str,
) -> Result<usize, StrOffsetError> {
    let b_hs = haystack.as_bytes();
    let ptr_hs = b_hs.as_ptr();
    let ptr_end_hs = unsafe { ptr_hs.offset(b_hs.len().try_into().unwrap()) };
    let b_needle = needle.as_bytes();
    let ptr_needle = b_needle.as_ptr();
    if ptr_needle < ptr_hs || ptr_needle > ptr_end_hs {
        return Err(StrOffsetError::NotContained);
    }
    let needle_offset = unsafe { ptr_needle.offset_from_unsigned(ptr_hs) };
    let before_needle = &b_hs[..needle_offset];
    #[cfg(debug_assertions)]
    let str_before = str::from_utf8(before_needle)?;
    #[cfg(not(debug_assertions))]
    let str_before = unsafe { str::from_utf8_unchecked(before_needle) };
    Ok(str_before.len())
}

pub(crate) fn split_prefix<'a>(s: &'a str, prefix: &'_ str) -> Option<(&'a str, &'a str)> {
    if prefix.len() > s.len() {
        return None;
    }
    let ret @ (maybe_prefix, _) = s.split_at(prefix.len());
    if prefix == maybe_prefix {
        Some(ret)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn str_offset_same() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[..];
        assert_eq!(unsafe { str_offset_unchecked(string, other) }?, 0);
        Ok(())
    }
    #[test]
    fn str_offset_inside() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[2..4];
        assert_eq!(unsafe { str_offset_unchecked(string, other) }?, 2);
        Ok(())
    }
    #[test]
    fn str_offset_inside_empty() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[2..2];
        assert_eq!(other, "");
        assert_eq!(unsafe { str_offset_unchecked(string, other) }?, 2);
        Ok(())
    }
    #[test]
    fn str_offset_last() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[string.len()..];
        assert_eq!(
            unsafe { str_offset_unchecked(string, other) }?,
            string.len()
        );
        Ok(())
    }
    #[test]
    fn str_offset_before() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[3..];
        assert!(matches!(
            unsafe { str_offset_unchecked(other, string) },
            Err(StrOffsetError::NotContained)
        ));
        Ok(())
    }
    #[test]
    fn str_offset_after() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[0..2];
        let string = &string[3..];
        assert_eq!(other, "My");
        assert_eq!(string, "string");
        assert!(matches!(
            unsafe { str_offset_unchecked(other, string) },
            Err(StrOffsetError::NotContained)
        ));
        Ok(())
    }
    #[test]
    fn str_offset_after_but_not_contained() -> Result<(), StrOffsetError> {
        let string = "My string";
        let other = &string[0..3];
        let string = &string[3..];
        assert_eq!(other, "My ");
        assert_eq!(string, "string");
        assert_eq!(unsafe { str_offset_unchecked(other, string) }?, other.len());
        Ok(())
    }
}
