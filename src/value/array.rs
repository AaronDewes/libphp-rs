use std::{fmt::Display, marker::PhantomData, ptr::NonNull};

use crate::sys::{
    HashTable, _zend_new_array, zend_array_count, zend_hash_add, zend_hash_get_current_data_ex,
    zend_hash_get_current_key_type_ex, zend_hash_get_current_key_zval_ex,
    zend_hash_move_forward_ex, zend_hash_next_index_insert, zend_string, zval,
    HASH_KEY_NON_EXISTENT, HT_MIN_SIZE,
};

use super::{string::create_zend_str, Value};

pub struct Array<'lifetime> {
    ptr: NonNull<HashTable>,
    _lifetime: PhantomData<&'lifetime ()>,
}

impl<'lifetime> Array<'lifetime> {
    pub fn new() -> Self {
        Self::with_capacity(HT_MIN_SIZE)
    }

    pub fn with_capacity(capacity: u32) -> Self {
        unsafe {
            let ptr = _zend_new_array(capacity);

            Self {
                ptr: NonNull::new_unchecked(ptr),
                _lifetime: Default::default(),
            }
        }
    }

    pub fn len(&self) -> usize {
        unsafe { zend_array_count(self.ptr.as_ptr()) as usize }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> ArrayIter {
        ArrayIter::new(unsafe { self.ptr.as_ref() })
    }

    pub unsafe fn insert(&mut self, key: &str, value: &mut Value) {
        unsafe {
            zend_hash_add(self.ptr.as_ptr(), create_zend_str(key), value.as_mut_ptr());
        }
    }

    pub unsafe fn insert_with_raw_key(&mut self, key: *mut zend_string, value: &mut Value) {
        unsafe {
            zend_hash_add(self.ptr.as_ptr(), key, value.as_mut_ptr());
        }
    }

    pub fn push(&mut self, value: &mut Value) {
        unsafe {
            zend_hash_next_index_insert(self.ptr.as_ptr(), value.as_mut_ptr());
        }
    }
}

pub struct ArrayIter<'a> {
    ptr: &'a HashTable,
    idx: u64,
    pos: u32,
}

impl<'a> ArrayIter<'a> {
    pub fn new(ptr: &'a HashTable) -> Self {
        Self {
            ptr,
            idx: 0,
            pos: 0,
        }
    }
}

pub enum ArrayKey {
    Int(i64),
    String(String),
}

impl Display for ArrayKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(i) => write!(f, "{}", i),
            Self::String(s) => write!(f, "{}", s),
        }
    }
}

impl<'a> Iterator for ArrayIter<'a> {
    type Item = (u64, ArrayKey, Value);

    fn next(&mut self) -> Option<Self::Item> {
        let key_type = unsafe {
            zend_hash_get_current_key_type_ex(
                self.ptr as *const HashTable as *mut HashTable,
                &mut self.pos,
            )
        };

        if key_type == HASH_KEY_NON_EXISTENT {
            return None;
        }

        let mut key = zval::default();

        unsafe {
            zend_hash_get_current_key_zval_ex(
                self.ptr as *const HashTable as *mut HashTable,
                &mut key,
                &mut self.pos,
            )
        };

        let key = Value::new_maybe_gc(NonNull::new(&mut key).unwrap());

        let value = Value::new_maybe_gc(NonNull::new(unsafe {
            zend_hash_get_current_data_ex(
                self.ptr as *const HashTable as *mut HashTable,
                &mut self.pos,
            )
        }).unwrap());

        let item = match key.is_int() {
            true => (self.idx, ArrayKey::Int(key.to_int()), value),
            false => (self.idx, ArrayKey::String(key.to_string()), value),
        };

        unsafe {
            zend_hash_move_forward_ex(
                self.ptr as *const HashTable as *mut HashTable,
                &mut self.pos,
            );
        }

        self.idx += 1;

        Some(item)
    }
}

impl<'a> From<&'a mut HashTable> for Array<'a> {
    fn from(value: &'a mut HashTable) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(value) },
            _lifetime: Default::default(),
        }
    }
}
