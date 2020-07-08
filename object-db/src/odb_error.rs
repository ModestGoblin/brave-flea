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
pub enum ODBError {
    BadDatabaseVersion,
}

impl ODBError {
    fn as_str(&self) -> &'static str {
        match *self {
            ODBError::BadDatabaseVersion => "The version number of this database file is not recognized by this version of Brave Flea.",
        }
    }
}

impl fmt::Display for ODBError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", self.as_str())
    }
}

impl error::Error for ODBError {
    fn description(&self) -> &str {
        self.as_str()
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}
