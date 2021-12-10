use std::ops::{Deref, DerefMut};

pub struct TailDropVec<T> {
    vec: Vec<T>,
}

impl<T> Drop for TailDropVec<T> {
    fn drop(&mut self) {
        loop {
            if let None = self.vec.pop() {
                break;
            }
        }
    }
}

impl<T> From<Vec<T>> for TailDropVec<T> {
    fn from(vec: Vec<T>) -> Self {
        TailDropVec { vec }
    }
}

impl<T> Deref for TailDropVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<T> DerefMut for TailDropVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vec
    }
}
