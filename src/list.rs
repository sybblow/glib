// Copyright 2013-2015, The Rust-GNOME Project Developers.
// See the COPYRIGHT file at the top-level directory of this distribution.
// Licensed under the MIT license, see the LICENSE file or <http://opensource.org/licenses/MIT>

// FIXME: @jeremyletang implements the new index traits when it's available

use libc::c_void;
use std::mem;
use std::ops::Index;
use std::iter::{FromIterator, IntoIterator};
use std::marker::PhantomData;
use ffi;

use glib_container::GlibContainer;

pub struct List<T> {
    pointer: *mut ffi::C_GList,
    _marker: PhantomData<T>
}

pub struct Elem<'a, T: 'a> {
    pointer: *mut ffi::C_GList,
    _marker: PhantomData<&'a T>
}

pub struct RevElem<'a, T: 'a> {
    pointer: *mut ffi::C_GList,
    _marker: PhantomData<&'a T>
}

impl<T> List<T> {
    pub fn new() -> List<T> {
        List {
            pointer: ::std::ptr::null_mut(),
            _marker: PhantomData
        }
    }

    pub fn from_vec(values: Vec<T>) -> List<T> {
        FromIterator::from_iter(values.into_iter())
    }

    pub fn from_slice(values: &[T]) -> List<T> where T: Clone {
        let v: Vec<T> = values.iter().map(|x| (*x).clone()).collect();
        FromIterator::from_iter(v.into_iter())
    }

    pub fn append(&mut self, data: T) {
        unsafe {
            self.pointer = ffi::g_list_append(self.pointer, mem::transmute(Box::new(data)));
        }
    }

    pub fn prepend(&mut self, data: T) {
        unsafe {
            self.pointer = ffi::g_list_prepend(self.pointer, mem::transmute(Box::new(data)));
        }
    }

    pub fn nth(&self, n: u32) -> &T {
        unsafe {
            mem::transmute::<*mut c_void, &T>(ffi::g_list_nth_data(self.pointer, n))
        }
    }

    pub fn last(&self) -> &T {
        let elem = unsafe { ffi::g_list_last(self.pointer) };
        unsafe { mem::transmute::<*mut c_void, &T>((*elem).data)}
    }

    pub fn first(&self) -> &T {
        let elem = unsafe { ffi::g_list_first(self.pointer) };
        unsafe { mem::transmute::<*mut c_void, &T>((*elem).data)}
    }

    pub fn insert(&mut self, data: T, position: i32) {
        unsafe {
            self.pointer = ffi::g_list_insert(self.pointer, mem::transmute(Box::new(data)), position);
        }
    }

    pub fn concat(&mut self, list: List<T>) {
        unsafe {
            ffi::g_list_concat(self.pointer, list.unwrap());
        }
    }

    pub fn reverse(&mut self) {
        unsafe {
            self.pointer = ffi::g_list_reverse(self.pointer);
        }
    }

    pub fn iter(&self) -> Elem<T> {
        Elem {
            pointer: unsafe { ffi::g_list_first(self.pointer) },
            _marker: PhantomData
        }
    }

    pub fn rev_iter(&self) -> RevElem<T> {
        RevElem {
            pointer: unsafe { ffi::g_list_last(self.pointer) },
            _marker: PhantomData
        }
    }

    pub fn len(&self) -> usize {
        unsafe { ffi::g_list_length(self.pointer) as usize }
    }

    pub fn clear(&mut self) {
        unsafe {
            ffi::g_list_free(self.pointer)
        }
    }

    pub fn extend<It: IntoIterator<Item=T>>(&mut self, it: It) {
        for elem in it {
            self.append(elem);
        }
    }
}

impl<T> Index<usize> for List<T> {
    type Output = T;

    fn index<'a>(&'a self, _rhs: usize) -> &'a T {
        self.nth(_rhs as u32)
    }
}

impl<'a, T> Iterator for Elem<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.pointer.is_null() {
            None
        } else {
            let ret = unsafe { mem::transmute::<*mut c_void, &T>((*self.pointer).data)};
            unsafe { self.pointer = (*self.pointer).next; }
            Some(ret)
        }
    }
}

impl<'a, T> Iterator for RevElem<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<&'a T> {
        if self.pointer.is_null() {
            None
        } else {
            let ret = unsafe { mem::transmute::<*mut c_void, &T>((*self.pointer).data)};
            unsafe { self.pointer = (*self.pointer).prev; }
            Some(ret)
        }
    }
}

impl<T> FromIterator<T> for List<T> {
    fn from_iter<It: IntoIterator<Item=T>>(it: It) -> List<T> {
        let mut new_list = List::new();
        new_list.extend(it);
        new_list
    }
}

impl<T> Clone for List<T> {
    fn clone(&self) -> List<T> {
        unsafe {
            GlibContainer::wrap(ffi::g_list_copy(self.pointer))
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        unsafe { ffi::g_list_free(self.pointer); }
    }
}

impl<T> GlibContainer<*mut ffi::C_GList> for List<T> {
    fn wrap(pointer: *mut ffi::C_GList) -> List<T> {
        List {
            pointer: pointer,
            _marker: PhantomData
        }
    }

    fn unwrap(&self) -> *mut ffi::C_GList {
        self.pointer
    }
}

#[cfg(test)]
mod bench{
    use super::List;
    extern crate test;
    use self::test::Bencher;

    #[bench]
    fn bench_collect_into(b: &mut test::Bencher) {
        let v: &[i32] = &[0; 128];
        b.iter(|| {
            List::from_slice(v);
        })
    }

    #[bench]
    fn bench_prepend(b: &mut test::Bencher) {
        let v: &[i32] = &[0; 128];
        let mut m: List<i32> = List::from_slice(v);
        b.iter(|| {
            m.prepend(0);
        })
    }

    #[bench]
    fn bench_prepend_empty(b: &mut test::Bencher) {
        let mut m: List<i32> = List::new();
        b.iter(|| {
            m.prepend(0);
        })
    }

    #[bench]
    fn bench_append(b: &mut test::Bencher) {
        let v: &[i32] = &[0; 128];
        let mut m: List<i32> = List::from_slice(v);
        b.iter(|| {
            m.append(0);
        })
    }

    #[bench]
    fn bench_append_empty(b: &mut test::Bencher) {
        let mut m: List<i32> = List::new();
        b.iter(|| {
            m.append(0);
        })
    }

    #[bench]
    fn bench_iter(b: &mut test::Bencher) {
        let v: &[i32] = &[0; 128];
        let m: List<i32> = List::from_slice(v);
        b.iter(|| {
            assert!(m.iter().count() == 128);
        })
    }
    #[bench]
    fn bench_iter_rev(b: &mut test::Bencher) {
        let v: &[i32] = &[0; 128];
        let m: List<i32> = List::from_slice(v);
        b.iter(|| {
            assert!(m.rev_iter().count() == 128);
        })
    }

}
