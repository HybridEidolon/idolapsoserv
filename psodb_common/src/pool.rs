//! Asynchronous connection pool.

use super::Backend;
use super::Result;
use super::error::Error;

use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};

/// A connection pool of Backend instances that can be safely moved over thread boundaries and
/// called asynchronously.
pub struct Pool {
    backends: Vec<Arc<Mutex<Box<Backend>>>>,
    counter: AtomicUsize
}

impl Pool {
    /// Fetch the next connection to use (not necessarily available). This minimizes the chance
    /// for blocking over the returned Backend, but does not completely eliminate it. Holders of
    /// a reference to the Backend should try to minimize time spent holding on the Mutex guard.
    pub fn get_connection(&self) -> Result<Arc<Mutex<Box<Backend>>>> {
        let i = self.counter.fetch_add(1, Ordering::SeqCst);
        match self.backends.get(i % self.backends.len()) {
            Some(b) => Ok(b.clone()),
            None => Err(Error::Other("unknown".to_string(), None))
        }
    }

    /// Creates a new connection pool, making the given number of clones of the base Backend.
    /// An error is returned if it fails to clone as many as requested.
    pub fn new(connections: usize, base: &mut Backend) -> Result<Pool> {
        let mut be = Vec::with_capacity(connections);
        for _ in 0..connections {
            let c = match base.try_clone() {
                Ok(b) => b,
                Err(e) => return Err(e)
            };
            be.push(Arc::new(Mutex::new(c)));
        }

        Ok(Pool {
            backends: be,
            counter: AtomicUsize::new(0)
        })
    }
}

unsafe impl Send for Pool {}
unsafe impl Sync for Pool {}
