use spin::lock_api::{Mutex, MutexGuard};

struct Locked<T> {
  inner: Mutex<T>,
}

impl<T> Locked<T> {
  fn lock(&self) -> MutexGuard<'_, T> {
    self.inner.lock()
  }
}
