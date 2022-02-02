use std::future::Future;
use std::pin::Pin;

pub(crate) struct PinnedQueue<T: ?Sized>(Vec<Pin<Box<T>>>);

impl<T: ?Sized> Default for PinnedQueue<T> {
    fn default() -> Self {
        Self::new()
    }
}

unsafe impl<T: ?Sized> Send for PinnedQueue<T> {}
unsafe impl<T: ?Sized> Sync for PinnedQueue<T> {}

impl<T: ?Sized> PinnedQueue<T> {
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn current_mut(&mut self) -> Option<&mut Pin<Box<T>>> {
        self.0.get_mut(0)
    }

    pub(crate) fn pop(&mut self) -> Option<Pin<Box<T>>> {
        if !self.0.is_empty() {
            Some(self.0.remove(0))
        } else {
            None
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Out> PinnedQueue<dyn Future<Output = Out>> {
    pub(crate) fn push_future(&mut self, future: impl Future<Output = Out> + 'static) {
        self.0.push(Box::pin(future) as _);
    }
}
