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

use crate::variable::*;
use db;

#[derive(Debug)]
pub enum Value {
    Uninitialized,
    NoValue,
    CharValue(char),
    IntValue(i32),
    BinaryValue(Vec<u8>),
    BooleanValue(bool),
    DateValue(std::time::SystemTime),
    DoubleValue(f32),
    StringValue(&'static str),
    ExternalValue,
    DiskValue(db::DBAddress),
}

#[derive(Debug)]
pub struct ValueRecord {
    value: Value,
    is_tmp_data: bool,
    is_tmp_stack: bool,
    is_formal_val: bool,
    is_disk_val: bool,
}

impl ValueRecord {
    pub fn new() -> Self {
        Self {
            value: Value::Uninitialized,
            is_tmp_data: false,
            is_tmp_stack: false,
            is_formal_val: false,
            is_disk_val: false,
        }
    }
}
