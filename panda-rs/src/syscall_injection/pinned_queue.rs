use parking_lot::{const_mutex, MappedMutexGuard, Mutex, MutexGuard};
use std::future::Future;
use std::pin::Pin;

use crate::prelude::*;

pub(crate) struct PinnedQueue<T: ?Sized>(Mutex<Vec<(target_ulong, Pin<Box<T>>)>>);

unsafe impl<T: ?Sized> Send for PinnedQueue<T> {}
unsafe impl<T: ?Sized> Sync for PinnedQueue<T> {}

impl<T: ?Sized> PinnedQueue<T> {
    pub(crate) const fn new() -> Self {
        Self(const_mutex(Vec::new()))
    }

    pub(crate) fn current(&self) -> Option<MappedMutexGuard<'_, (target_ulong, Pin<Box<T>>)>> {
        let lock = self.0.lock();
        if !lock.is_empty() {
            Some(MutexGuard::map(lock, |queue| queue.get_mut(0).unwrap()))
        } else {
            None
        }
    }

    pub(crate) fn pop(&self) -> Option<(target_ulong, Pin<Box<T>>)> {
        let mut lock = self.0.lock();
        if !lock.is_empty() {
            Some(lock.remove(0))
        } else {
            None
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.0.lock().len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Out> PinnedQueue<dyn Future<Output = Out>> {
    pub(crate) fn push_future(
        &self,
        asid: target_ulong,
        future: impl Future<Output = Out> + 'static,
    ) {
        self.0.lock().push((asid, Box::pin(future) as _));
    }
}
