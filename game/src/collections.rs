use crate::data::DataError;
use serde::{Deserialize, Serialize};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
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

#[derive(Debug, Serialize, Deserialize)]
pub enum DictionaryError {
    KeyNotFound { key: String },
    NameNotFound { name: String },
}

impl<K, T> Dictionary<K, T>
where
    K: Debug + Hash + Eq,
{
    pub fn len(&self) -> usize {
        self.keys.len()
    }

    pub fn insert(&mut self, key: K, name: String, kind: T) {
        let kind = Shared::new(kind);
        self.keys.insert(key, kind.clone());
        self.strings.insert(name.to_string(), kind);
    }

    pub fn get(&self, key: K) -> Result<Shared<T>, DictionaryError>
    where
        K: Debug,
    {
        self.keys
            .get(&key)
            .cloned()
            .ok_or(DictionaryError::KeyNotFound {
                key: format!("{:?}", key),
            })
    }

    pub fn find(&self, name: &str) -> Result<Shared<T>, DictionaryError> {
        self.strings
            .get(name)
            .cloned()
            .ok_or(DictionaryError::NameNotFound {
                name: name.to_string(),
            })
    }

    pub fn find2(&self, name: &String) -> Result<Shared<T>, DictionaryError> {
        self.strings
            .get(name)
            .cloned()
            .ok_or(DictionaryError::NameNotFound {
                name: name.to_string(),
            })
    }

    pub fn find_by(&self, row: &rusqlite::Row, index: &str) -> Result<Shared<T>, DataError> {
        let name: String = row.get(index)?;
        let kind = self.find(&name)?;
        Ok(kind)
    }

    pub fn get_by<C>(
        &self,
        row: &rusqlite::Row,
        index: &str,
        constructor: C,
    ) -> Result<Shared<T>, DataError>
    where
        C: Fn(usize) -> K,
    {
        let key: usize = row.get(index)?;
        let key = constructor(key);
        let kind = self.get(key)?;
        Ok(kind)
    }
}

#[derive(Default, Clone, Copy, Eq, Hash, PartialEq)]
pub struct Sequence {
    value: usize,
}

impl Sequence {
    pub fn one<C, T>(&mut self, constructor: C) -> T
    where
        C: Fn(usize) -> T,
    {
        self.value += 1;
        constructor(self.value)
    }

    pub fn many<C, T, const N: usize>(&mut self, constructor: C) -> [T; N]
    where
        T: Copy,
        C: Fn(usize) -> T,
    {
        let mut result = [constructor(0); N];
        for i in 0..N {
            self.value += 1;
            result[i] = constructor(self.value);
        }
        result
    }

    pub fn many_vec<C, T>(&mut self, count: usize, constructor: C) -> Vec<T>
    where
        T: Copy,
        C: Fn(usize) -> T,
    {
        let mut result = vec![constructor(0); count];
        for i in 0..count {
            self.value += 1;
            result[i] = constructor(self.value);
        }
        result
    }

    pub fn set(&mut self, value: usize) {
        self.value = value;
    }

    pub fn register(&mut self, id: usize) {
        if id > self.value {
            self.value = id
        }
    }

    pub fn introduce(&self) -> Sequence {
        Sequence { value: self.value }
    }
}

pub fn trust<T>(value: &mut T) -> TrustedRef<T> {
    TrustedRef::from(value)
}

pub struct TrustedRef<T> {
    ptr: *mut T,
}

impl<T> TrustedRef<T> {
    pub fn from(value: &mut T) -> Self {
        Self {
            ptr: value as *mut _,
        }
    }

    pub fn get_unsafe(&self) -> &T {
        unsafe { &*self.ptr }
    }

    pub fn get_mut_unsafe(&self) -> &mut T {
        unsafe { &mut *self.ptr }
    }
}

impl<T> Deref for TrustedRef<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get_unsafe()
    }
}

impl<T> DerefMut for TrustedRef<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut_unsafe()
    }
}
