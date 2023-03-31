use crate::data::DataError;
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

#[derive(Debug, bincode::Encode, bincode::Decode)]
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

//
// pub trait DictionaryQuery<K, T, C> {
//     fn find_by(&self, row: &rusqlite::Row, index: &str) -> Result<Shared<T>, DataError>;
//     fn get_by(
//         &self,
//         row: &rusqlite::Row,
//         index: &str,
//         constructor: C,
//     ) -> Result<Shared<T>, DataError>;
// }
//
// impl<K, C, T> DictionaryQuery<K, T, C> for Dictionary<K, T>
//     where
//         K: Debug + Hash + Eq,
//         C: Fn(usize) -> K,
// {
//     fn find_by(&self, row: &rusqlite::Row, index: &str) -> Result<Shared<T>, DataError> {
//         let name: String = row.get(index)?;
//         let kind = self.find(&name)?;
//         Ok(kind)
//     }
//
//     fn get_by(
//         &self,
//         row: &rusqlite::Row,
//         index: &str,
//         constructor: C,
//     ) -> Result<Shared<T>, DataError> {
//         let key: usize = row.get(index)?;
//         let key = constructor(key);
//         let kind = self.get(key)?;
//         Ok(kind)
//     }
// }
