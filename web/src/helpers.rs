use std::{cell::LazyCell, marker::PhantomData, sync::atomic::AtomicUsize};

use leptos::{
    attr::{
        Attr, Loading,
        custom::{CustomAttr, custom_attribute},
    },
    ev, html, logging,
    prelude::*,
    tachys::view::iterators::StaticVec,
};
use leptos_router::{
    LazyRoute, MatchNestedRoutes, any_nested_route::IntoAnyNestedRoute as _, hooks::use_location,
};

/// Zero Width Non-Joiner
///
/// Can be used to block phone number detection for browsers that ignore the meta tag
pub(crate) const ZWNJ: char = '\u{200C}';

#[cfg_attr(not(feature = "client-side"), allow(unused))]
pub(crate) trait ScopedTimeout {
    fn set_scoped_timeout(&self, timeout: std::time::Duration, action: impl 'static + FnOnce());
    fn request_scoped_animation_frame(&self, action: impl 'static + FnOnce());
    fn request_idle_callback(
        &self,
        fallback: std::time::Duration,
        max_wait: Option<std::time::Duration>,
        action: impl 'static + FnOnce(),
    );
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

    fn request_idle_callback(
        &self,
        fallback: std::time::Duration,
        max_wait: Option<std::time::Duration>,
        action: impl 'static + FnOnce(),
    ) {
        let action = ArcRwSignal::new(Some(action));
        let owner = self.downgrade();
        let idle_timeout = request_idle_callback_with_handle({
            let action = action.clone();
            let owner = owner.clone();
            move || {
                let owner = owner.clone();
                let mut action = action.write();
                if let Some(inner) = action.take()
                    && let Some(owner) = owner.upgrade()
                {
                    owner.with(inner);
                }
            }
        });

        if let Ok(idle_timeout) = idle_timeout {
            if let Some(max_wait) = max_wait {
                self.set_scoped_timeout(max_wait, move || {
                    let mut action = action.write();
                    idle_timeout.cancel();
                    if let Some(inner) = action.take() {
                        inner()
                    } else {
                        action.untrack()
                    }
                });
            }
        } else {
            self.set_scoped_timeout(fallback, move || {
                let mut action = action.write();
                if let Some(inner) = action.take() {
                    inner();
                } else {
                    action.untrack();
                }
            })
        }
    }
}

pub(crate) fn idle_preload<T: LazyRoute>() {
    #[cfg(feature = "client-side")]
    if let Some(owner) = Owner::current() {
        use crate::helpers::ScopedTimeout as _;
        owner.request_idle_callback(std::time::Duration::from_secs(2), None, || {
            leptos::task::spawn_local_scoped(T::preload())
        });
    }
}

#[inline(always)]
pub(crate) fn scoped_style() -> CustomAttr<&'static str, bool> {
    custom_attribute("scoped", true)
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

#[cfg_attr(not(feature = "client-side"), allow(unused))]
impl<I: 'static + Iterator, F: 'static + Fn(<I as Iterator>::Item)> IntervalIterator<I, F> {
    #[allow(dead_code)]
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
    pub(crate) fn into_scoped_animation_timeout(mut self) {
        let Some(i) = self.it.next() else { return };
        let Some(owner) = self.owner.upgrade() else {
            logging::warn!("Lost iterator owner");
            return;
        };
        let interval = self.interval;
        owner.set_scoped_timeout(interval, move || {
            if let Some(owner) = self.owner.upgrade() {
                owner.request_scoped_animation_frame(move || {
                    (self.action)(i);
                    self.into_scoped_animation_timeout();
                });
            }
        });
    }
}

#[inline(always)]
pub fn into_static_str<T>(source: T) -> &'static str
where
    T: Into<&'static str>,
{
    source.into()
}

#[cfg_attr(not(feature = "client-side"), allow(unused))]
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
        IntervalIterator {
            it: self,
            interval,
            action,
            owner: Owner::current().unwrap().downgrade(),
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
    x: impl Fn() -> (T, Option<R>),
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
            if let Some(r) = r {
                provide_context(context);
                r
            } else {
                provide_context(context.clone());
                f(context)
            }
        };
        if in_root {
            root_context.with(make_context)
        } else {
            make_context()
        }
    }
}

#[derive(Clone)]
struct InterestedOwners<T>(RwSignal<Vec<WeakOwner>>, PhantomData<T>);
impl<T: Clone + Sync + Send + 'static> InterestedOwners<T> {
    fn singleton() -> RwSignal<Vec<WeakOwner>> {
        once_by_type(
            true,
            || (Self(RwSignal::new(vec![]), PhantomData), None),
            |InterestedOwners(owners, _)| owners,
        )
    }
}
pub(crate) fn has_interested_owners<T: Clone + Sync + Send + 'static>() -> Memo<bool> {
    let owners = InterestedOwners::<T>::singleton();
    Memo::new(move |_| {
        owners.track();
        let mut owners = owners.write();
        let was_empty = owners.is_empty();
        let living_owners = owners
            .iter()
            .filter_map(|o| o.upgrade())
            .map(|x| x.downgrade());
        *owners = living_owners.collect();
        if owners.is_empty() == was_empty {
            owners.untrack();
        }
        !owners.is_empty()
    })
}

pub(crate) fn register_interested_owner<T: Clone + Sync + Send + 'static>() {
    let owner = Owner::current().unwrap();
    let interested = InterestedOwners::<T>::singleton();
    let mut owners = interested.write();
    if owners
        .iter()
        .find(|x| x.upgrade().as_ref() == Some(&owner))
        .is_none()
    {
        if let Some(parent) = owner.parent() {
            parent.with(|| Owner::on_cleanup(move || interested.notify()))
        }
        owners.push(owner.downgrade());
    }
}

/// Same as `<noscript>`, except that this element is automatically removed after hydration. This
/// is done to circumvent a bug where <noscript><style></style></noscript> changes the style of the
/// document when scripts are available.
#[component]
pub(crate) fn NoScript(children: ChildrenFn) -> impl IntoView {
    let noscript_ref = NodeRef::<html::Noscript>::new();
    Effect::new(move || {
        let Some(node) = noscript_ref.get() else {
            return;
        };
        let Some(parent) = node.parent_node() else {
            return;
        };
        _ = parent.remove_child(&node);
    });
    view! { <noscript node_ref=noscript_ref>{(children)()}</noscript> }.into_inner()
}

/// This element will be replaced in runtime by the children of this type. Note that these children
/// obviously cannot use Leptos interative features.
#[component]
pub(crate) fn NoWasm(children: ChildrenFn) -> impl IntoView {
    #[derive(Clone, Copy)]
    struct NoWasmScriptLoaded;
    let init_script = once_by_type(
        true,
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
                Some(Some(view! { <script nonce=use_nonce()>{script}</script> })),
            )
        },
        |_| None,
    );

    let template_elem = Suspend::new(async move {
        let inner_html = if cfg!(feature = "ssr") {
            let children = (children)().resolve().await;
            Oco::Owned(children.to_html())
        } else {
            Oco::Borrowed("")
        };
        view! {
            <template
                class:wasm-fallback
                {..custom_attribute("shadowrootclone", ())}
                inner_html=inner_html
            />
        }
        .into_inner()
    });

    view! {
        <Suspense>{template_elem}</Suspense>
        {init_script}
    }
}

#[inline(always)]
pub(crate) fn img_def() -> (Attr<Loading, &'static str>,) {
    (Attr(Loading, "lazy"),)
}

#[derive(Clone)]
struct FootnoteInner {
    uid: usize,
    name: ArcSignal<Oco<'static, str>>,
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
            (FootnotesHolder((active, visible)), Some((active, visible)))
        },
        |FootnotesHolder((active, visible))| (active, visible),
    )
}

type AbbrList = Vec<(usize, ArcSignal<Oco<'static, str>>)>;

fn abbrs() -> (ReadSignal<AbbrList>, WriteSignal<AbbrList>) {
    #[cfg_attr(debug_assertions, derive(Debug))]
    #[derive(Clone)]
    struct AllAbbrs<T>(ReadSignal<T>, WriteSignal<T>);
    once_by_type(
        true,
        || {
            let (reader, writer) = signal::<Vec<(usize, ArcSignal<Oco<str>>)>>(vec![]);
            (AllAbbrs(reader, writer), Some((reader, writer)))
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
    let is_current = move |name: ArcSignal<Oco<'static, str>>| {
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
                                        name.clone(),
                                        (*children)(),
                                    ))
                                    .collect::<Vec<_>>()
                            })
                    }
                    key=|(name, _)| name.get()
                    let((name, inner))
                >
                    <div id=name.clone() aria-current=is_current(name.clone())>
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
        name: ArcSignal::derive(footnote_name.clone()),
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
        <span>
            <a
                on:click=on_click
                id=footnote_source.clone()
                aria-describedby=footnote_name.clone()
                aria-current=move || is_current(footnote_source.clone())
                class="footnote-link"
                href=move || format!("#{}", footnote_name())
            />
        </span>
    }
    .into_inner()
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
    #[prop(into)] mut title: Oco<'static, str>,
    #[prop(into, optional)] suffix: Option<Oco<'static, str>>,
    #[prop(optional, default = false)] no_expand: bool,
    children: TypedChildrenMut<T>,
) -> impl IntoView {
    let (read_abbrs, write_abbrs) = abbrs();
    static FOOT_IDX: AtomicUsize = AtomicUsize::new(1);
    let uid = FOOT_IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    title.upgrade_inplace();

    write_abbrs.update({
        let title = title.clone();
        move |abbrs| abbrs.push((uid, ArcSignal::derive(move || title.clone())))
    });

    Owner::on_cleanup(move || {
        write_abbrs.update(move |abbrs| {
            if let Some((index, _)) = abbrs.iter().enumerate().find(|(_, (id, _))| *id == uid) {
                abbrs.remove(index);
            }
        });
    });

    let first_of_abbr = {
        let title = title.clone();
        move || {
            if no_expand {
                false
            } else {
                let abbr = title.clone();
                read_abbrs.with(move |abbrs| {
                    abbrs
                        .iter()
                        .find(|(_, name)| name.with(|name| *name == abbr))
                        .map(|x| x.0)
                        == Some(uid)
                })
            }
        }
    };
    view! {
        <abbr
            title={
                let (title, suffix) = (title, suffix).clone();
                move || {
                    if let Some(s) = &suffix {
                        Oco::Owned(format!("{title}{s}"))
                    } else {
                        title.clone()
                    }
                }
            }
            class:first-of-abbr=first_of_abbr
        >
            {children.into_inner()}
            {suffix.clone()}
        </abbr>
    }
    .into_inner()
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
