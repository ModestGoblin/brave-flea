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

use crate::error::*;
use crate::string_utils::*;
use std::convert::TryInto;

pub const WINDOW_INFO_SIZE: u32 = 62;

#[derive(Debug)]
struct Rect {
    pub top: u16,
    pub left: u16,
    pub bottom: u16,
    pub right: u16,
}

#[derive(Debug)]
pub struct WindowInfo {
    window_rect: Rect,
    font_string: String,
    font_number: u16,
    font_size: u16,
    font_style: u16,
    is_hidden: bool,
    is_unused: bool,
}

impl WindowInfo {
    pub fn new(buffer: &[u8]) -> Result<WindowInfo> {
        let top = u16::from_be_bytes(buffer[0..2].try_into()?);
        let left = u16::from_be_bytes(buffer[2..4].try_into()?);
        let bottom = u16::from_be_bytes(buffer[4..6].try_into()?);
        let right = u16::from_be_bytes(buffer[6..8].try_into()?);
        let font_string = read_pascal_string(&buffer[8..41]); // 33 byte pascal string starting at 8

        // ignore bytes 42 - 43
        let font_number = 0;
        let font_size = u16::from_be_bytes(buffer[44..46].try_into()?);
        let font_style = u16::from_be_bytes(buffer[46..48].try_into()?);
        // ignore bytes 48 - 51
        let is_hidden = buffer[52] != 0;
        let is_unused = buffer[53] != 0;

        Ok(WindowInfo {
            window_rect: Rect {
                top,
                left,
                bottom,
                right,
            },
            font_string,
            font_number,
            font_size,
            font_style,
            is_hidden,
            is_unused,
        })
    }
}
