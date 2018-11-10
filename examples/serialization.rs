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
        let _ = File::create("data.json")
            .map_err(|e| {
                println!("could not create data file: {:?}", e);
            })
            .and_then(|f| {
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
