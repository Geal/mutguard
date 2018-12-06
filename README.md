# MutGuard

[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Build Status](https://travis-ci.org/Geal/mutguard.svg?branch=master)](https://travis-ci.org/Geal/mutguard)
[![Coverage Status](https://coveralls.io/repos/Geal/mutguard/badge.svg?branch=master)](https://coveralls.io/r/Geal/mutguard?branch=master)
[![Crates.io Version](https://img.shields.io/crates/v/mut_guard.svg)](https://crates.io/crates/mut_guard)
[![Documentation](https://docs.rs/mut_guard/badge.svg)](https://docs.rs/mut_guard)

this library allows you to call a function after
some data has been mutably borrowed.

*Note*: that function will be called from a `Drop` handler

## Use cases

### Invariant checks

It can be used to enforce invariant: every time a `&mut` is obtained,
be it from the element's method definition, or from external code
accessing public members directly, the invariant check will run and
verify the data is correct.

```rust
extern crate mut_guard;
use mut_guard::*;

#[derive(Debug)]
struct LessThan20(pub u8);

impl Guard for LessThan20 {
  fn finish(&mut self) {
    assert!(self.0 <= 20, "invariant failed, internal value is too large: {}", self.0);
  }
}

fn main() {
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
  // This code will panic with the following message:
  // 'invariant failed, internal value is too large: 30'
  val.guard().0 = 30;
}
```

### Logging

Since the guard will be called every time there's a mutable access, we can log the changes
there:

```rust
# extern crate mut_guard;
# use mut_guard::*;
#
# fn main() {
  let v = Vec::new();

  // the wrap methods allows us to Specifiesy a closure instead of manually
  // implementing Guard
  let mut iv = MutGuard::wrap(v, |ref mut vec| {
    println!("vector content is now {:?}", vec);
  });

  iv.guard().push(1);
  // prints "vector content is now [1]"

  iv.guard().push(2);
  // prints "vector content is now [1, 2]"

  iv.guard().push(3);
  // prints "vector content is now [1, 2, 3]"
# }
```

### Serialization

The guard function could be used to store the element to a file after every change.

```rust
#[macro_use]
extern crate serde_derive;
extern crate mut_guard;
extern crate serde;
extern crate serde_json;

use mut_guard::*;
use std::fs::File;

#[derive(Serialize, Debug)]
struct Data {
    pub a: u32,
    pub s: String,
    pub v: Vec<u32>,
}

impl Guard for Data {
    fn finish(&mut self) {
        File::create("data.json")
            .map_err(|e| {
                println!("could not create data file");
            })
            .and_then(|mut f| {
                serde_json::to_writer(f, self).map_err(|e| {
                    println!("could not serialize data: {:?}", e);
                })
            });
    }
}

fn main() {
    let mut data = MutGuard::new(Data {
        a: 0,
        s: "hello".to_string(),
        v: vec![1, 2],
    });

    data.guard().s = "Hello world".to_string();
    // data.json was created and now contains:
    // {"a":0,"s":"Hello world","v":[1,2]}
}
```
