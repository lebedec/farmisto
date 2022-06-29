use std::cell::{RefCell, RefMut};
use std::fmt::{Debug, Formatter};
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
