use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

/// Provides safe access to game assets data.
///
/// Usage:
/// - possibility to hot reload in development mode
/// - sharing data between game objects
#[derive(Clone)]
pub struct Asset<T> {
    data: Arc<RefCell<T>>,
}

impl<T> Deref for Asset<T> {
    type Target = T;

    /// # Safety
    /// Dereference of raw pointer is safe because of
    /// underlying data update possible in single thread method Assets::update only
    /// (before any dereference attempts).
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.data.as_ptr() }
    }
}

impl<T> From<Arc<RefCell<T>>> for Asset<T> {
    fn from(data: Arc<RefCell<T>>) -> Self {
        Self { data }
    }
}

pub trait AssetMap<T> {
    fn publish(&mut self, name: &str, data: T) -> Asset<T>;
}

impl<T> AssetMap<T> for HashMap<String, Asset<T>> {
    fn publish(&mut self, name: &str, data: T) -> Asset<T> {
        let asset = Asset::from(data);
        self.insert(name.to_string(), asset.share());
        asset
    }
}

impl<T> From<T> for Asset<T> {
    fn from(data: T) -> Self {
        Self {
            data: Arc::new(RefCell::new(data)),
        }
    }
}

impl<T: Sized> Asset<T> {
    #[inline]
    pub fn update(&mut self, data: T) {
        let mut this = self.data.borrow_mut();
        *this = data;
    }

    pub fn share(&self) -> Asset<T> {
        Self {
            data: self.data.clone(),
        }
    }
}
