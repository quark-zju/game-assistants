use std::env;
use std::fmt;
use std::hash::Hash;
use std::ops::Index;
use std::ops::IndexMut;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

static VERBOSE: AtomicUsize = AtomicUsize::new(0);

pub fn cached_bool_env(cached: &AtomicUsize, env_name: &'static str) -> bool {
    let v = cached.load(Ordering::Relaxed);
    match v {
        0 => {
            let b = env::var_os(env_name).is_some();
            let v = match b {
                false => 2,
                true => 3,
            };
            cached.store(v, Ordering::Relaxed);
            b
        }
        2 => false,
        _ => true,
    }
}

/// Whether run under debug mode.
pub fn is_debug() -> bool {
    static DEBUG: AtomicUsize = AtomicUsize::new(0);
    cached_bool_env(&DEBUG, "D")
}

/// Whether run under verbose mode.
pub fn is_verbose() -> bool {
    static VERBOSE: AtomicUsize = AtomicUsize::new(0);
    cached_bool_env(&VERBOSE, "V")
}

#[macro_export]
macro_rules! dprintln {
    ($($t:tt)*) => {
        if crate::util::is_debug() {
            eprintln!($($t)*);
        }
    }
}
#[macro_export]
macro_rules! or {
    ($e:expr, $($s:tt)*) => {
        match $e {
            Some(v) => v,
            None => $($s)*,
        }
    };
}

/// Small vector.
#[derive(Copy, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct NVec<T, const N: usize> {
    inner: [T; N],
    len: u8,
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for NVec<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries((0..self.len).map(|i| &self[i]))
            .finish()
    }
}

impl<T: Default + Copy, const N: usize> Default for NVec<T, N> {
    fn default() -> Self {
        Self {
            inner: [T::default(); N],
            len: 0,
        }
    }
}

impl<T, const N: usize> Index<u8> for NVec<T, N> {
    type Output = T;

    fn index(&self, index: u8) -> &Self::Output {
        assert!(index < self.len);
        &self.inner[index as usize]
    }
}

impl<T, const N: usize> IndexMut<u8> for NVec<T, N> {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        assert!(index < self.len);
        &mut self.inner[index as usize]
    }
}

impl<T, const N: usize> AsRef<[T]> for NVec<T, N> {
    fn as_ref(&self) -> &[T] {
        &self.inner[..self.len as usize]
    }
}

impl<T, const N: usize> NVec<T, N> {
    pub fn len(&self) -> u8 {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn push(&mut self, v: T) {
        self.inner[self.len as usize] = v;
        self.len += 1;
    }

    pub fn last(&self) -> Option<&T> {
        if self.len == 0 {
            None
        } else {
            Some(&self[self.len - 1])
        }
    }
}

impl<T: Copy + Default, const N: usize> NVec<T, N> {
    pub fn pop(&mut self) -> Option<T> {
        if self.len == 0 {
            None
        } else {
            self.len -= 1;
            let v = self.inner[self.len as usize];
            self.inner[self.len as usize] = T::default();
            Some(v)
        }
    }
}
