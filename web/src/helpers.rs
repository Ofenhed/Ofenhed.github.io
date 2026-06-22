use std::sync::atomic::AtomicUsize;

use leptos::{
    attr::{Attr, Loading, custom::custom_attribute},
    ev,
    logging::log,
    prelude::*,
    tachys::view::iterators::StaticVec,
};
use leptos_meta::Script;
use leptos_router::{
    MatchNestedRoutes, any_nested_route::IntoAnyNestedRoute as _, components::Outlet,
    hooks::use_location,
};

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
            log!("Lost iterator owner");
            return;
        };
        let interval = self.interval;
        owner.set_scoped_timeout(interval, move || {
            (self.action)(i);
            self.into_scoped_timeout();
        });
    }
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

#[component]
pub(crate) fn AddContext<T: Send + Sync + 'static>(
    context: T,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    #[derive(Clone, Copy)]
    struct OutletRendered;
    if let Some(writer) = use_context::<WriteSignal<_>>() {
        writer.set(context)
    }
    let children = children.map(|children| {
        Owner::current().map(move |x| {
            x.child().with(move || {
                provide_context(OutletRendered);
                let child = children();
                child
            })
        })
    });
    let outlet = if use_context::<OutletRendered>().is_none() {
        Some(view! { <Outlet /> }.into_inner())
    } else {
        None
    };
    view! {
        {outlet}
        {children}
    }
}

pub(crate) fn once_by_type<T: Clone + Send + Sync + 'static, R>(
    x: T,
    f: impl Fn() -> R,
) -> Option<R> {
    let root = {
        let mut owner = Owner::current().unwrap();
        while let Some(parent) = owner.parent() {
            owner = parent;
        }
        owner
    };
    if let Some(_) = root.use_context_bidirectional::<T>() {
        None
    } else {
        provide_context(x);
        Some(f())
    }
}

#[component]
pub(crate) fn NoWasm(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    #[derive(Clone, Copy)]
    struct NoWasmScriptLoaded;
    let init_script = once_by_type(NoWasmScriptLoaded, || {
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
        view! { <Script>{script}</Script> }
    });

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

pub(crate) fn provide_footnote_context() {
    _ = footnotes(false);
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
#[cfg_attr(debug_assertions, derive(Debug))]
#[derive(Clone)]
struct FootnotesHolder(FootnotesInner);

fn footnotes(warn: bool) -> FootnotesInner {
    let FootnotesHolder(signal) = use_context().unwrap_or_else(|| {
        if warn {
            leptos::logging::warn!(
                "Footnotes not created in top context. Please use provide_footnote_context()."
            );
        }
        let footnotes = FootnotesHolder((RwSignal::new(None), RwSignal::new(vec![])));
        provide_context(footnotes.clone());
        footnotes
    });
    signal
}

macro_rules! scroll_into_view {
    ($element:ident) => {
        #[cfg(feature = "client-side")]
        {
            $element.scroll_into_view_with_scroll_into_view_options(&{
                let opts = web_sys::ScrollIntoViewOptions::new();
                opts.set_behavior(web_sys::ScrollBehavior::Smooth);
                opts
            })
        }
        #[cfg(not(feature = "client-side"))]
        {
            _ = $element;
        }
    };
}

#[component]
pub(crate) fn Footnotes() -> impl IntoView {
    let (active, footnotes) = footnotes(true);
    let current_hash = use_location().hash;
    let is_current = move |name| {
        move || {
            active.with(|x| x.as_ref().map(|x| *x == name).unwrap_or(false))
                || current_hash.with(|hash| hash.strip_prefix('#').unwrap_or(hash) == name)
        }
    };
    let on_click = move |_source_id: Oco<'static, str>| {
        move |e: ev::MouseEvent| {
            active.set(None);
            if let Some(footnote) = document().get_element_by_id(&*_source_id) {
                scroll_into_view!(footnote);
                e.prevent_default();
            }
        }
    };

    let return_link = move |target_id: Oco<'static, str>| {
        view! {
            <a
                on:click=on_click(target_id.clone())
                class="footnote-return-link"
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
                                        name.get(),
                                        (*children)(),
                                    ))
                                    .collect::<Vec<_>>()
                            })
                    }
                    key=|(name, _)| name.clone()
                    let((name, inner))
                >
                    <div id=name.clone() aria-current=is_current(name.clone())>
                        <div>{inner}</div>
                        {return_link(Oco::Owned(format!("{}-source", name)))}
                    </div>
                </For>
            </footer>
        </Show>
    }
}

#[component]
pub(crate) fn Footnote(
    #[prop(into, optional)] name: Signal<Option<Oco<'static, str>>>,
    children: ChildrenFn,
) -> impl IntoView {
    let (active, footnotes) = footnotes(true);
    static FOOT_IDX: AtomicUsize = AtomicUsize::new(1);
    let uid = FOOT_IDX.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    let footnote_name = move || {
        name.get()
            .unwrap_or_else(|| Oco::Owned(format!("footer-{uid}")))
    };

    let my_footnote = FootnoteInner {
        uid,
        name: Signal::derive(move || footnote_name()),
        children,
    };

    Owner::on_cleanup({
        move || {
            footnotes.update(move |x| {
                if let Some((idx, _)) = x
                    .iter()
                    .enumerate()
                    .find(move |(_, FootnoteInner { uid: elem_uid, .. })| uid == *elem_uid)
                {
                    x.remove(idx);
                }
            });
        }
    });

    footnotes.update(move |x| {
        x.push(my_footnote);
    });

    let on_click = move |e: ev::MouseEvent| {
        let name = footnote_name();
        if let Some(footnote) = document().get_element_by_id(&name) {
            active.set(Some(name));
            scroll_into_view!(footnote);
            e.prevent_default();
        }
    };

    view! {
        <a
            on:click=on_click
            id=move || format!("{}-source", footnote_name())
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
