// This module is mostly copied from `https://github.com/museun/mock_instant`
// The reason is that the repository has not been updated in a while.

use core::sync::atomic::{AtomicU64, Ordering};
use core::time::Duration;

pub static TIME: AtomicU64 = AtomicU64::new(0);

pub fn with_time(d: impl Fn(&mut Duration)) {
    let nanos = TIME.load(Ordering::SeqCst);
    let mut dur = Duration::from_nanos(nanos);
    d(&mut dur);
    TIME.store(dur.as_nanos() as u64, Ordering::SeqCst);
}

pub fn get_time() -> Duration {
    Duration::from_nanos(TIME.load(Ordering::SeqCst))
}

/// A Mock clock
///
/// This uses shared mutable state to have a deterministic clock.
#[derive(Copy, Clone)]
pub struct MockClock;

// impl core::fmt::Debug for MockClock {
//     fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
//         f.debug_struct("MockClock").field("time", &Self::time()).finish()
//     }
// }

impl MockClock {
    /// Set the internal clock to this 'Duration'
    pub fn set_time(time: Duration) {
        TIME.store(time.as_nanos() as u64, Ordering::SeqCst);
    }

    /// Advance the internal clock by this 'Duration'
    pub fn advance(time: Duration) {
        TIME.fetch_add(time.as_nanos() as u64, Ordering::SeqCst);
    }

    /// Get the current duration
    pub fn time() -> Duration {
        get_time()
    }
}

#[no_mangle]
pub extern "C" fn __mock_time_set_ns(nanos: u64) {
    MockClock::set_time(Duration::from_nanos(nanos));
}

#[no_mangle]
pub extern "C" fn __mock_time_advance_ns(nanos: u64) {
    MockClock::advance(Duration::from_nanos(nanos));
}

/// A simple deterministic Instant wrapped around a modifiable Duration
///
/// This uses shared state as the 'wall clock' that is configurable via
/// the `MockClock`
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Instant(Duration);

impl Instant {
    pub fn now() -> Instant {
        Self(MockClock::time())
    }

    pub fn elapsed(&self) -> Duration {
        MockClock::time() - self.0
    }
}
