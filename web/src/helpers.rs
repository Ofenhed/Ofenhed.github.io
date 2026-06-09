use leptos::{
    attr::{Attr, Loading, custom::custom_attribute},
    logging::log,
    prelude::*,
    tachys::view::iterators::StaticVec,
};
use leptos_meta::Script;
use leptos_router::{
    MatchNestedRoutes, any_nested_route::IntoAnyNestedRoute as _, components::Outlet,
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

#[component]
pub(crate) fn NoWasm(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    #[derive(Clone, Copy)]
    struct NoWasmScriptLoaded;
    let load_init_script = {
        let root = {
            let mut owner = Owner::current().unwrap();
            while let Some(parent) = owner.parent() {
                owner = parent;
            }
            owner
        };
        if let Some(NoWasmScriptLoaded) = root.use_context_bidirectional() {
            None
        } else {
            provide_context(NoWasmScriptLoaded);
            Some(())
        }
    };
    let init_script = load_init_script.map(|()| {
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
