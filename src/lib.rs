//! # MutGuard
//!
//! this library allows you to call a function after
//! some data has been mutably borrowed.
//!
//! *Note*: that function will be called from a `Drop` handler
//!
//! ## Use cases
//!
//! ### Invariant checks
//!
//! It can be used to enforce invariant: every time a `&mut` is obtained,
//! be it from the element's method definition, or from external code
//! accessing public members directly, the invariant check will run and
//! verify the data is correct.
//!
//! ```rust,should_panic
//! extern crate mut_guard;
//! use mut_guard::*;
//!
//! #[derive(Debug)]
//! struct LessThan20(pub u8);
//!
//! impl Guard for LessThan20 {
//!   fn finish(&mut self) {
//!     assert!(self.0 <= 20, "invariant failed, internal value is too large: {}", self.0);
//!   }
//! }
//!
//! fn main() {
//!   let mut val = MutGuard::new(LessThan20(0));
//!
//!   //"val: 0"
//!   println!("val: {:?}", *val);
//!
//!   // this would fail because MutGuard does not implement DerefMut directly
//!   //val.0 = 10;
//!
//!   // use the guard() method to get a `&mut LessThan20`
//!   val.guard().0 = 10;
//!
//!   //"val: 10"
//!   println!("val: {:?}", *val);
//!
//!   // once the value returned by guard() is dropped, the invariant will be checked
//!   // This code will panic with the following message:
//!   // 'invariant failed, internal value is too large: 30'
//!   val.guard().0 = 30;
//! }
//! ```
//!
//! ### Logging
//!
//! Since the guard will be called every time there's a mutable access, we can log the changes
//! there:
//!
//! ```rust
//! # extern crate mut_guard;
//! # use mut_guard::*;
//! #
//! # fn main() {
//!   let v = Vec::new();
//!
//!   // the wrap methods allows us to Specifiesy a closure instead of manually
//!   // implementing Guard
//!   let mut iv = MutGuard::wrap(v, |ref mut vec| {
//!     println!("vector content is now {:?}", vec);
//!   });
//!
//!   iv.guard().push(1);
//!   // prints "vector content is now [1]"
//!
//!   iv.guard().push(2);
//!   // prints "vector content is now [1, 2]"
//!
//!   iv.guard().push(3);
//!   // prints "vector content is now [1, 2, 3]"
//! # }
//! ```
//!
//! ### Serialization
//!
//! The guard function could be used to store the element to a file after every change
//!
//! ```rust
//! #[macro_use]
//! extern crate serde_derive;
//! extern crate mut_guard;
//! extern crate serde;
//! extern crate serde_json;
//!
//! use mut_guard::*;
//! use std::fs::File;
//!
//! #[derive(Serialize, Debug)]
//! struct Data {
//!     pub a: u32,
//!     pub s: String,
//!     pub v: Vec<u32>,
//! }
//!
//! impl Guard for Data {
//!     fn finish(&mut self) {
//!         File::create("data.json")
//!             .map_err(|e| {
//!                 println!("could not create data file");
//!             })
//!             .and_then(|mut f| {
//!                 serde_json::to_writer(f, self).map_err(|e| {
//!                     println!("could not serialize data: {:?}", e);
//!                 })
//!             });
//!     }
//! }
//!
//! fn main() {
//!     let mut data = MutGuard::new(Data {
//!         a: 0,
//!         s: "hello".to_string(),
//!         v: vec![1, 2],
//!     });
//!
//!     data.guard().s = "Hello world".to_string();
//!     // data.json was created and now contains:
//!     // {"a":0,"s":"Hello world","v":[1,2]}
//! }
//! ```
//!
use std::ops::{Deref, DerefMut, Drop};

/// stores an inner element that must implement the `Guard` trait,
/// and forbids mutable borrows except going through its `guard()` method.
pub struct MutGuard<T> {
    inner: T,
}

impl<T> Deref for MutGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

/// Specifies a method that will be called after every time an element
/// protected by a `Mutguard` will be mutably borrowed
pub trait Guard {
    fn finish(&mut self);
}

impl<T: Guard> MutGuard<T> {
    pub fn new(inner: T) -> MutGuard<T> {
        MutGuard { inner }
    }

    /// call this method to get mutable access to the underlying element
    pub fn guard(&mut self) -> MutGuardBorrow<T> {
        MutGuardBorrow { inner: self }
    }

    /// returns the wrapped element, consuming the MutGuard
    pub fn into_inner(self) -> T {
        self.inner
    }
}

impl<'a, T> MutGuard<MutGuardWrapper<'a, T>> {
    /// This method automatically generates a `Guard` implementation that will
    /// call `f` after every time the inner element is mutably borrowed
    pub fn wrap<F>(inner: T, f: F) -> MutGuard<MutGuardWrapper<'a, T>>
    where
        F: 'a + for<'r> FnMut(&'r mut T),
    {
        let wrapper = MutGuardWrapper {
            inner,
            f: Box::new(f),
        };
        MutGuard::new(wrapper)
    }
}

/// Structure returned by the `MutGuard::guard()`. when this is dropped, it
/// will call the `Guard::finish()` method of the wrapped element
pub struct MutGuardBorrow<'a, T: 'a + Guard> {
    inner: &'a mut MutGuard<T>,
}

impl<'a, T: Guard> Deref for MutGuardBorrow<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner.inner
    }
}

impl<'a, T: Guard> DerefMut for MutGuardBorrow<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner.inner
    }
}

impl<'a, T: Guard> Drop for MutGuardBorrow<'a, T> {
    fn drop(&mut self) {
        self.inner.inner.finish();
    }
}

/// `Guard` implementation returned by `MutGuard::wrap()`
pub struct MutGuardWrapper<'a, T> {
    inner: T,
    f: Box<'a + FnMut(&mut T)>,
}

impl<'a, T: 'a> MutGuardWrapper<'a, T> {
    pub fn new<F>(inner: T, f: F) -> MutGuardWrapper<'a, T>
    where
        F: 'a + for<'r> FnMut(&'r mut T),
    {
        MutGuardWrapper {
            inner,
            f: Box::new(f),
        }
    }
}

impl<'a, T> Guard for MutGuardWrapper<'a, T> {
    fn finish(&mut self) {
        (self.f)(&mut self.inner);
    }
}

impl<'a, T> Deref for MutGuardWrapper<'a, T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.inner
    }
}

impl<'a, T> DerefMut for MutGuardWrapper<'a, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct Bank {
        accounts: Vec<i32>,
    }

    impl Bank {
        pub fn new(accounts: Vec<i32>) -> Bank {
            Bank { accounts }
        }

        pub fn transfer(&mut self, account1: usize, account2: usize, amount: i32) {
            self.accounts[account1] -= amount;
            self.accounts[account2] += amount;
        }
    }

    impl Guard for Bank {
        fn finish(&mut self) {
            assert!(
                self.accounts.iter().any(|v| *v < 0),
                "accounts should not become negative"
            );
        }
    }

    #[test]
    #[should_panic(expected = "accounts should not become negative")]
    fn invariant_bank() {
        let mut ibank = MutGuard::new(Bank::new(vec![10, 0, 20, 50]));

        println!("bank: {:?}", *ibank);

        {
            ibank.guard().transfer(0, 1, 5);
            println!("bank: {:?}", *ibank);

            ibank.guard().transfer(2, 3, 30);
            println!("bank: {:?}", *ibank);
        }
    }

    #[test]
    fn bank() {
        let mut bank = Bank::new(vec![10, 0, 20, 50]);
        println!("bank: {:?}", bank);

        bank.transfer(0, 1, 5);
        println!("bank: {:?}", bank);

        bank.transfer(2, 3, 30);
        // now accounts are [5, 5, -10, 80]
        println!("bank: {:?}", bank);
    }

    #[test]
    fn count_access() {
        let mut counter = 0;
        let v = Vec::new();

        {
            let mut iv = MutGuard::wrap(v, |_| counter += 1);

            iv.guard().push(1);
            iv.guard().push(2);
            iv.guard().push(3);
            assert_eq!(iv[0], 1);
            assert_eq!(iv[1], 2);
            assert_eq!(iv[2], 3);
        }

        assert_eq!(counter, 3);
    }

    #[test]
    #[should_panic]
    fn less_than() {
        #[derive(Debug)]
        struct LessThan20(pub u8);

        impl Guard for LessThan20 {
            fn finish(&mut self) {
                assert!(
                    self.0 <= 20,
                    "invariant failed, internal value is too large: {}",
                    self.0
                );
            }
        }

        let mut val = MutGuard::new(LessThan20(0));

        //"val: 0"
        println!("val: {:?}", *val);

        // this would fail because MutGuard does not implement DerefMut directly
        //val.0 = 10;

        // use the guard() method to get a `&mut LessThan20`
        val.guard().0 = 10;

        //"val: 10"
        println!("val: {:?}", *val);

        // once the value returned by guard() is dropped, the invariant will be checked
        // we get the message "panicked at 'invariant failed, internal value is too large: 30'"
        val.guard().0 = 30;
    }
}
