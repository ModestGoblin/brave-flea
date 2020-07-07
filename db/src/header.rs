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

use crate::error::Result;
use std::convert::TryInto;

pub const HEADER_SIZE: u32 = 8;

pub struct DBHeader {
    pub is_free: bool,
    pub size: u32,
    pub variance: u32,
}

impl DBHeader {
    pub fn new(buffer: &[u8; 8]) -> Result<Self> {
        let size_and_free = u32::from_be_bytes(buffer[0..=3].try_into()?);
        let variance = u32::from_be_bytes(buffer[4..=7].try_into()?);

        let is_free = (size_and_free & 0x80000000) == 0x80000000;
        let size = size_and_free & 0x7FFFFFFF;

        Ok(Self {
            is_free,
            size,
            variance,
        })
    }
}
