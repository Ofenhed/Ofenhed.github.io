use leptos::{logging::log, prelude::*};

pub trait ScopedTimeout {
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

pub fn set_scoped_timeout(timeout: std::time::Duration, action: impl 'static + FnOnce()) {
    let Some(owner) = Owner::current() else {
        return;
    };
    owner.set_scoped_timeout(timeout, action)
}
pub fn request_scoped_animation_frame(action: impl 'static + FnOnce()) {
    let Some(owner) = Owner::current() else {
        return;
    };
    owner.request_scoped_animation_frame(action)
}

pub struct IntervalIterator<I, F> {
    it: I,
    interval: std::time::Duration,
    action: F,
    owner: WeakOwner,
}

impl<I: 'static + Iterator, F: 'static + Fn(<I as Iterator>::Item)> IntervalIterator<I, F> {
    pub fn into_scoped_timeout(mut self) {
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

pub trait IntoIntervalIterator<F>
where
    Self: Iterator + Sized,
{
    #[must_use]
    fn on_interval(self, interval: std::time::Duration, action: F) -> IntervalIterator<Self, F>;
}

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
