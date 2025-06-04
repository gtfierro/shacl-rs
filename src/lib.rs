#![allow(dead_code, unused_variables)]
mod types;
mod components;
mod shape;
mod context;
mod named_nodes;

use components::Component;
use shape::Shape;
use types::ID;
use std::collections::HashMap;

pub struct Store {
    shape_lookup: HashMap<ID, Shape>,
    component_lookup: HashMap<ID, Component>,
}


pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
