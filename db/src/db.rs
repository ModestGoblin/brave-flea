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

use crate::available_node::*;
use crate::db_error::*;
use crate::error::*;
use crate::header::*;

const DB_VERSION_NUMBER: u8 = 6;
const DB_FIRST_VERSION_WITH_CACHED_SHADOW_AVAIL_LIST: u8 = 6;
const VIEW_COUNT: usize = 3;
const DATABASE_RECORD_SIZE: u32 = 88;
const DIRTY_MASK: u16 = 0x0001;
const MAJOR_VERSION_MASK: u8 = 0x00f0;
// const MINOR_VERSION_MASK: u8 = 0x000f;

pub type DBAddress = u32;
const DB_ADDRESS_SIZE: usize = std::mem::size_of::<DBAddress>();
const NIL_DB_ADDRESS: DBAddress = 0;

#[derive(Debug)]
pub struct Database {
    system_id: u8,
    version_number: u8,
    avail_list: DBAddress,
    is_dirty: bool,
    views: [DBAddress; VIEW_COUNT],
    release_stack: Vec<DBAddress>,
    file: fs::File,
    long_version_major: u16,
    long_version_minor: u16,
    avail_list_block: DBAddress,
    avail_list_shadow: Vec<AvailableNodeShadow>,
    is_read_only: bool,
}

const AVAILABLE_NODE_SHADOW_SIZE: usize = 8;

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
            is_dirty: false,
            views: [0; VIEW_COUNT],
            release_stack: vec![],
            file,
            long_version_major: 0,
            long_version_minor: 0,
            avail_list_block: 0,
            avail_list_shadow: vec![],
            is_read_only: read_only,
        };

        let mut buffer: [u8; DATABASE_RECORD_SIZE as usize] = [0; DATABASE_RECORD_SIZE as usize];
        db.read(0, DATABASE_RECORD_SIZE, &mut buffer)?;

        db.system_id = buffer[0]; // byte 0
        db.version_number = buffer[1]; // byte 1
        db.avail_list = u32::from_be_bytes(buffer[2..=5].try_into()?); // bytes 2-5

        // ignore bytes 6-7 (short oldfnumdatabase)
        let flags = u16::from_be_bytes(buffer[8..=9].try_into()?); // bytes 8-9
        db.is_dirty = (flags & DIRTY_MASK) != 0;

        // bytes 10-21
        for i in 0..VIEW_COUNT {
            db.views[i] = u32::from_be_bytes(buffer[10 + (i * 4)..14 + (i * 4)].try_into()?);
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
            db.is_dirty = true;
        }

        db.shadow_avail_list()?;

        Ok(db)
    }

    fn shadow_avail_list(&mut self) -> Result<()> {
        if self.avail_list_block != NIL_DB_ADDRESS {
            if self.read_shadow_avail_list()? {
                return Ok(());
            }
        }

        let db_eof = self.get_eof()?;
        self.avail_list_shadow.clear();

        let mut next_avail_adr = self.avail_list;

        while next_avail_adr != NIL_DB_ADDRESS {
            let next_avail_node = self.read_available_node(next_avail_adr)?;
            if !next_avail_node.is_free || next_avail_adr + next_avail_node.size > db_eof {
                self.avail_list_shadow.clear();
                return Err(Error::from(DBError::FreeList));
            }

            self.avail_list_shadow.push(AvailableNodeShadow {
                address: next_avail_adr,
                size: next_avail_node.size,
            });

            next_avail_adr = next_avail_node.next_node;
        }

        Ok(())
    }

    fn read_shadow_avail_list(&mut self) -> Result<bool> {
        self.avail_list_shadow.clear();
        let db_eof = self.get_eof()?;

        if self.avail_list_block == NIL_DB_ADDRESS {
            // There is no shadow avall list
            return Ok(false);
        }

        let buffer = self.read_block(self.avail_list_block)?;
        let avail_shadow_count = buffer.len() / AVAILABLE_NODE_SHADOW_SIZE;

        for i in 0..avail_shadow_count {
            let address = u32::from_be_bytes(
                buffer[(i * AVAILABLE_NODE_SHADOW_SIZE)..4 + (i * AVAILABLE_NODE_SHADOW_SIZE)]
                    .try_into()?,
            );
            let size = u32::from_be_bytes(
                buffer[4 + (i * AVAILABLE_NODE_SHADOW_SIZE)..8 + (i * AVAILABLE_NODE_SHADOW_SIZE)]
                    .try_into()?,
            );

            if address != NIL_DB_ADDRESS {
                self.avail_list_shadow
                    .push(AvailableNodeShadow { address, size });
            }
        }

        if self.avail_list_shadow.is_empty() {
            return Err(Error::from(DBError::InconsistentAvailList));
        }

        // Test consistency of caches shadow avail list
        let first_address = self.avail_list_shadow[0].address;
        if first_address != self.avail_list {
            self.avail_list_shadow.clear();
            return Err(Error::from(DBError::InconsistentAvailList));
        }

        if first_address != NIL_DB_ADDRESS {
            let first_node = self.read_available_node(first_address)?;

            if !first_node.is_free || first_address + first_node.size > db_eof {
                self.avail_list_shadow.clear();
                return Err(Error::from(DBError::InconsistentAvailList));
            }
        }

        Ok(true)
    }

    fn read_available_node(&mut self, address: DBAddress) -> Result<AvailableNode> {
        let header = self.read_header(address)?;

        let mut buffer = [0; DB_ADDRESS_SIZE];
        self.read(address + HEADER_SIZE, DB_ADDRESS_SIZE as u32, &mut buffer)?;
        let next_node = DBAddress::from_be_bytes(buffer);

        Ok(AvailableNode {
            is_free: header.is_free,
            size: header.size,
            next_node,
        })
    }

    fn read_block(&mut self, address: DBAddress) -> Result<Vec<u8>> {
        if address == NIL_DB_ADDRESS {
            return Err(Error::from(DBError::InvalidAddress));
        }

        let header = self.read_header(address)?;

        let block_size = header.size - header.variance;

        if header.is_free {
            return Err(Error::from(DBError::FreeBlock));
        }

        let mut buffer = vec![0; block_size as usize];
        self.read(address + HEADER_SIZE, block_size, &mut buffer)?;

        Ok(buffer)
    }

    fn read_header(&mut self, address: DBAddress) -> Result<DBHeader> {
        let mut buffer = [0; HEADER_SIZE as usize];
        self.read(address, HEADER_SIZE, &mut buffer)?;

        DBHeader::new(&buffer)
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

    fn get_eof(&mut self) -> Result<DBAddress> {
        let old_pos = self.file.seek(io::SeekFrom::Current(0))?;
        let eof = self.file.seek(io::SeekFrom::End(0))?;

        // Avoid seeking a third time when we were already at the end of the
        // stream.
        if old_pos != eof {
            self.file.seek(io::SeekFrom::Start(old_pos))?;
        }

        Ok(eof as DBAddress)
    }
}
