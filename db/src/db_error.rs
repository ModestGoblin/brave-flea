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

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum DBError {
    WrongVersion,
    FreeBlock,
    FreeList,
    InconsistentAvailList,
    AssignFreeBlock,
    FileSize,
    ReleaseFreeBlock,
    ReleaseInvalidBlock,
    MergeInvalidBlock,
    InvalidAddress,
}

impl DBError {
    fn as_str(&self) -> &'static str {
        match *self {
            DBError::WrongVersion => "File was created by an incompatible version of this program.",
            DBError::FreeBlock => "Internal database error: attempted to read a free block. Try to Save a Copy and relaunch with the new database.",
            DBError::FreeList => "This database has a damaged free list. Use the Save a Copy command to create a new, compacted database.",
            DBError::InconsistentAvailList => "This database has an inconsistent list of free blocks. Use the Save a Copy command to create a new, compacted database.",
            DBError::AssignFreeBlock => "Internal database error: attempted to assign to a free block. Try to Save a Copy and relaunch with the new database.",
            DBError::FileSize => "Internal database error: attempted to grow the file beyond the maximum database size.",
            DBError::ReleaseFreeBlock => "Internal database error: attempted to release a free block.",
            DBError::ReleaseInvalidBlock => "Internal database error: attempted to release an invalid block.",
            DBError::MergeInvalidBlock => "Internal database error: attempted to merge with an invalid block.",
            DBError::InvalidAddress => "Attempted to read from an invalid dbaddress."
        }
    }
}

impl fmt::Display for DBError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl error::Error for DBError {
    fn description(&self) -> &str {
        self.as_str()
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
