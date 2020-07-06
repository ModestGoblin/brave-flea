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

use std::cmp;
use std::convert::TryInto;
use std::fs;
use std::io;
use std::io::prelude::*;

use crate::error::*;
use crate::db_error::*;

const DB_VERSION_NUMBER: u8 = 6;
const DB_FIRST_VERSION_WITH_CACHED_SHADOW_AVAIL_LIST: u8 = 6;
const VIEW_COUNT: usize = 3;
const DATABASE_RECORD_SIZE: u32 = 88;
const DIRTY_MASK: u16 = 0x0001;
const MAJOR_VERSION_MASK: u8 = 0x00f0;
// const MINOR_VERSION_MASK: u8 = 0x000f;

type DBAddress = u32;

const NIL_DB_ADDRESS: DBAddress = 0;

#[derive(Debug)]
pub struct Database {
    system_id: u8,
    version_number: u8,
    avail_list: DBAddress,
    dirty: bool,
    views: [DBAddress; VIEW_COUNT],
    release_stack: Vec<DBAddress>,
    file: fs::File,
    long_version_major: u16,
    long_version_minor: u16,
    avail_list_block: DBAddress,
    avail_list_shadow: Vec<AvailableNodeShadow>,
    read_only: bool,
}

#[derive(Debug)]
struct AvailableNodeShadow {
    pub address: DBAddress,
    pub size: u32,
}

impl Database {
    pub fn open_file(file: fs::File, read_only: bool) -> Result<Self> {
        let mut db = Self {
            system_id: 0,
            version_number: 0,
            avail_list: 0,
            dirty: false,
            views: [0; VIEW_COUNT],
            release_stack: vec![],
            file,
            long_version_major: 0,
            long_version_minor: 0,
            avail_list_block: 0,
            avail_list_shadow: vec![],
            read_only,
        };

        let mut buffer: [u8; DATABASE_RECORD_SIZE as usize] = [0; DATABASE_RECORD_SIZE as usize];
        db.read(0, DATABASE_RECORD_SIZE, &mut buffer)?;

        db.system_id = buffer[0]; // byte 0
        db.version_number = buffer[1]; // byte 1
        db.avail_list = u32::from_be_bytes(buffer[2..=5].try_into()?); // bytes 2-5
        // ignore bytes 6-7 (short oldfnumdatabase)
        let flags = u16::from_be_bytes(buffer[8..=9].try_into()?); // bytes 8-9
        db.dirty = (flags & DIRTY_MASK) != 0;

        for i in 0..VIEW_COUNT { // bytes 10-21
            db.views[i] = u32::from_be_bytes(buffer[(i * 4) + 10..((i+1)*4)+10].try_into()?);
        }

        // ignore bytes 22-25 (Handle releasestack)
        // ignore bytes 26-29 (long fnumdatabase)
        // ignore bytes 30-33 (long headerLength)
        db.long_version_major = u16::from_be_bytes(buffer[34..=35].try_into()?); // bytes 34-35
        db.long_version_minor = u16::from_be_bytes(buffer[36..=37].try_into()?); // bytes 36-37

        db.avail_list_block = u32::from_be_bytes(buffer[38..=41].try_into()?); // bytes 38-41
        // ignore bytes 42 - 57 (handlestream availlistshadow)
        // ignore byte 58 (boolean flreadonly)
        // ignore bytes 59 - 87 (growthspace)

        if db.version_number != DB_VERSION_NUMBER {
            if db.version_number & MAJOR_VERSION_MASK != DB_VERSION_NUMBER & MAJOR_VERSION_MASK {
                return Err(Error::from(DBError::WrongVersion));
            }

            if db.version_number < DB_FIRST_VERSION_WITH_CACHED_SHADOW_AVAIL_LIST {
                db.avail_list_block = NIL_DB_ADDRESS;
            }

            db.version_number = DB_VERSION_NUMBER;
            db.dirty = true;
        }

        db.shadow_avail_list()?;

        Ok(db)
    }

    fn shadow_avail_list(&mut self) -> Result<()> {
        // TODO:
        Ok(())
    }

    fn read(&mut self, address: DBAddress, byte_count: u32, buffer: &mut [u8]) -> Result<()> {
        self.seek(address)?;

        let max = cmp::max(buffer.len(), byte_count as usize);
        let actual_byte_count = self.file.read(&mut buffer[..max])?;

        if actual_byte_count < byte_count as usize {
            return Err(Error::from(io::Error::from(io::ErrorKind::UnexpectedEof)));
        }

        Ok(())
    }

    fn seek(&mut self, address: DBAddress) -> Result<()> {
        self.file.seek(io::SeekFrom::Start(address as u64))?;
        Ok(())
    }
}
