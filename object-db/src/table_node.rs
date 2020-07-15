/*
Brave Flea
Copyright (C) 2020  Ted C. Howard

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

use crate::value_record::*;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct TableNode {
    // next_node: Option<Rc<Box<TableNode>>>,
    // next_sorted_node: Option<Weak<Box<TableNode>>>,
    value: ValueRecord, // TODO: make value type
    dont_save: bool,
    is_locked: bool,
    is_protected: bool,
    is_unresolved_address: bool,
    is_dispose_when_unlocked: bool,
    locks_count: u8,
    hash_key: &'static str,
}

impl TableNode {
    pub fn new() -> Self {
        Self {
            value: ValueRecord::new(),
            dont_save: false,
            is_locked: false,
            is_protected: false,
            is_unresolved_address: false,
            is_dispose_when_unlocked: false,
            locks_count: 0,
            hash_key: "",
        }
    }
    fn test(&mut self, node: Rc<Box<TableNode>>) {
        // let node = TableNode {
        //     next_node: None,
        //     next_sorted_node: None,
        //     value: 0,
        //     dont_save: false,
        //     is_locked: false,
        //     is_protected: false,
        //     is_unresolved_address: false,
        //     is_dispose_when_unlocked: false,
        //     locks_count: 0,
        //     hash_key: [0; 255],
        // };

        // self.next_node = Some(Rc::clone(&node));
        // self.next_sorted_node = Some(Rc::downgrade(&node));
    }
}
