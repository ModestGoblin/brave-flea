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

use std::array;
use std::error;
use std::fmt;
use std::io;
use std::result;

use crate::db_error::DBError;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    DB(DBError),
    TryFromSlice(array::TryFromSliceError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Io(ref err) => write!(f, "IO error: {}", err),
            Error::DB(ref err) => write!(f, "DB error: {}", err),
            Error::TryFromSlice(ref err) => write!(f, "Array Error: {}", err),
        }
    }
}

#[allow(deprecated, deprecated_in_future)]
impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Io(ref err) => err.description(),
            Error::DB(ref err) => err.description(),
            Error::TryFromSlice(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        match *self {
            Error::Io(ref err) => Some(err),
            Error::DB(ref err) => Some(err),
            Error::TryFromSlice(ref err) => Some(err),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<DBError> for Error {
    fn from(err: DBError) -> Self {
        Error::DB(err)
    }
}

impl From<array::TryFromSliceError> for Error {
    fn from(err: array::TryFromSliceError) -> Self {
        Error::TryFromSlice(err)
    }
}
