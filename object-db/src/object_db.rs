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
use crate::odb_error::*;
use crate::string_utils::*;
use crate::table::*;
use crate::window_info::*;
use db;
use std::convert::TryInto;
use std::fs;

const ODB_VIEW: usize = 0;
const WINDOW_INFO_COUNT: usize = 6;

const IS_FLAG_DISABLED_MASK: u16 = 0x8000;
const IS_POPUP_DISABLED_MASK: u16 = 0x4000;
const IS_BIG_WINDOW_MASK: u16 = 0x2000;

#[derive(Debug)]
pub struct ObjectDB {
    db: db::Database,
    window_info: Vec<WindowInfo>,
    script_string: String,
    is_flag_disabled: bool,
    is_popup_disabled: bool,
    is_big_window: bool,
    root_table: Option<Table>,
}

impl ObjectDB {
    pub fn load_file(file: fs::File) -> Result<Self> {
        let mut db = db::Database::open_file(file, false)?;
        let address = db.get_view(ODB_VIEW);

        let mut buffer = [0; 2];
        db.read_block_into_buffer(address, 2, &mut buffer)?;
        let version_number = u16::from_be_bytes(buffer);

        let mut odb = Self {
            db,
            window_info: vec![],
            script_string: String::from(""),
            is_flag_disabled: false,
            is_popup_disabled: false,
            is_big_window: false,
            root_table: None,
        };

        // Brave Flea does not support ODB version 1 files
        if !(version_number == 2 || version_number == 3) {
            return Err(Error::from(ODBError::BadDatabaseVersion));
        }

        let buffer = odb.db.read_block(address)?;
        // let version_number = u16::from_be_bytes(buffer[0..2].try_into()?);
        let root_table_address = u32::from_be_bytes(buffer[2..6].try_into()?);

        // bytes 6 - 377
        let size = WINDOW_INFO_SIZE as usize;
        for i in 0..WINDOW_INFO_COUNT {
            odb.window_info.push(WindowInfo::new(
                &buffer[6 + (i * size)..(6 + size) + (i * size)],
            )?);
        }

        // TODO: fontier4root conversion

        let script_string_address = u32::from_be_bytes(buffer[378..382].try_into()?);
        let flags = u16::from_be_bytes(buffer[382..384].try_into()?);
        // let primary_agent_index = u16::from_be_bytes(buffer[384..386].try_into()?);

        // ignore bytes 386 - 441 (short waste[28])

        if script_string_address != db::NIL_DB_ADDRESS {
            let buffer = odb.db.read_block(script_string_address)?;
            odb.script_string = read_fixed_string(&buffer);
        }

        odb.is_flag_disabled = (flags & IS_FLAG_DISABLED_MASK) != 0;
        odb.is_popup_disabled = (flags & IS_POPUP_DISABLED_MASK) != 0;
        odb.is_big_window = (flags & IS_BIG_WINDOW_MASK) != 0;

        // TODO: load system table
        odb.load_system_table(root_table_address, false)?;

        // TODO: guest root logic

        Ok(odb)
    }

    fn load_system_table(&mut self, address: db::DBAddress, create: bool) -> Result<()> {
        let table = Table::load_system_table(&mut self.db, address)?;
        self.root_table = Some(table);
        Ok(())
    }
}
