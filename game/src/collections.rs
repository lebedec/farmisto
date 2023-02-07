use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::rc::Rc;

pub struct Shared<T> {
    inner: Rc<RefCell<T>>,
}

impl<T: Debug> Debug for Shared<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(self.deref(), f)
    }
}

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self {
            inner: Rc::new(RefCell::new(value)),
        }
    }

    #[inline]
    pub fn borrow_mut(&mut self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}

impl<T> Deref for Shared<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.inner.as_ptr() }
    }
}

pub struct Dictionary<K, T> {
    keys: HashMap<K, Shared<T>>,
    strings: HashMap<String, Shared<T>>,
}

impl<K, T> Default for Dictionary<K, T> {
    fn default() -> Self {
        Self {
            keys: HashMap::default(),
            strings: HashMap::default(),
        }
    }
}

impl<K, T> Dictionary<K, T>
where
    K: Hash + Eq,
{
    pub fn insert(&mut self, key: K, name: String, kind: T) {
        let kind = Shared::new(kind);
        self.keys.insert(key, kind.clone());
        self.strings.insert(name.to_string(), kind);
    }

    pub fn get(&self, key: K) -> Option<Shared<T>> {
        self.keys.get(&key).cloned()
    }

    pub fn find(&self, name: &str) -> Option<Shared<T>> {
        self.strings.get(name).cloned()
    }
}
