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

// #[macro_use]
// extern crate lazy_static;

use crate::error::*;
use crate::string_utils::*;
use crate::table_node::*;
use crate::variable::*;
use db::{DBAddress, Database, NIL_DB_ADDRESS};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::prelude::*;
use std::iter::Map;
use std::ops::{Add, Sub};
use std::rc::{Rc, Weak};
use std::time;

const NODE_BUCKET_COUNT: usize = 11;
// https://www.epochconverter.com/mac
const CLASSIC_MAC_EPOCH_OFFSET: time::Duration = time::Duration::from_secs(2082844800);

#[derive(Debug)]
pub struct Table {
    nodes: HashMap<String, TableNode>,
    sorted_keys: Vec<String>,
    // node_buckets: Vec<Option<Rc<Box<TableNode>>>>,
    // first_sorted_node: Option<Weak<Box<TableNode>>>,
    // prev_table: Option<Weak<Box<Table>>>,
    // parent_table: Option<Weak<Box<Table>>>,
    my_node: Option<Weak<Box<TableNode>>>,

    is_dirty: bool,
    is_locked: bool,
    is_window_open: bool,
    is_no_purge: bool,
    is_local_table: bool,
    is_chained: bool,
    is_dispose_when_unchained: bool,
    with_value_count: u8,
    is_verbs_require_window: bool,
    is_need_sort: bool,
    is_may_affect_display: bool,
    is_subs_dirty: bool,

    //TODO: long hashtablerefcon;
    //TODO: long lexicalrefcon;

    //TODO: struct tytableformats **hashtableformats;
    sort_order: u16,
    time_created: time::SystemTime,
    time_last_saved: time::SystemTime,

    //TODO: langvaluecallback valueroutine
    temp_stack_count: u16,
    //TODO: tyvaluerecord tmpstack []
}

impl Table {
    pub fn new() -> Self {
        let now = time::SystemTime::now();

        Self {
            nodes: HashMap::new(),
            sorted_keys: vec![],
            // node_buckets: vec![None; NODE_BUCKET_COUNT],
            // first_sorted_node: None,
            // prev_table: None,
            // parent_table: None,
            my_node: None,
            is_dirty: false,
            is_locked: false,
            is_window_open: false,
            is_no_purge: false,
            is_local_table: false,
            is_chained: false,
            is_dispose_when_unchained: false,
            with_value_count: 0,
            is_verbs_require_window: false,
            is_need_sort: false,
            is_may_affect_display: false,
            is_subs_dirty: false,
            sort_order: 0,
            time_created: now,
            time_last_saved: now,
            temp_stack_count: 0,
        }
    }

    fn sort_nodes(&mut self) {
        let mut keys: Vec<_> = self.nodes.keys().map(|s| s.clone()).collect();
        keys.sort_by(|a, b| a.cmp(b));
        self.sorted_keys = keys;
    }

    pub fn load_system_table(db: &mut Database, address: DBAddress) -> Result<Self> {
        if address == NIL_DB_ADDRESS {
            // TODO: start an empty table
        } else {
            let mut variable = Variable::<Table>::new_on_disk(db, address);
            variable.load_from_disk()?;

            if let VariableData::InMemory(mut tbl) = variable.data {
                tbl.data.sort_nodes();
                return Ok(tbl.data);
            }
        }

        Ok(Self::new())
    }
}

impl LoadFromBytes for Table {
    fn load_from_bytes(bytes: &[u8]) -> Result<Table> {
        let (packed_table, packed_formats) = split_buffer(bytes)?;

        let mut table = Table::new();

        table.unpack_table(packed_table)?;

        Ok(table)
    }
}

impl Table {
    fn unpack_table(&mut self, packed_table: &[u8]) -> Result<()> {
        let (records, strings) = split_buffer(packed_table)?;

        let mut index = 0;
        let header_size = 16;
        let mut header = DiskHeader::new(&records[index..(index + header_size)])?;
        index += header_size;
        let mut sorted = false;

        if header.version > 0 {
            self.sort_order = header.sort_order;
            let classic_mac_epoch = time::UNIX_EPOCH.sub(CLASSIC_MAC_EPOCH_OFFSET);
            self.time_created =
                classic_mac_epoch.add(time::Duration::from_secs(header.time_created as u64));
            self.time_last_saved =
                classic_mac_epoch.add(time::Duration::from_secs(header.time_last_saved as u64));

            if header.version == 2 {
                header.flags = 0;
            }

            sorted = true;
        } else {
            header.version = 0;
            header.flags = 0;
            let now = time::SystemTime::now();
            self.time_created = now;
            self.time_last_saved = now;
            index = 0;
        }

        let disk_symbol_size = 10;
        let chunks = records[index..].chunks(disk_symbol_size);

        for chunk in chunks {
            let mut rec = DiskSymbolRecord::new(chunk)?;
            if header.version < 2 {
                rec.version >>= 4;
            }

            let strings_index = u32::from_be_bytes(rec.data);
            let name = read_pascal_string(&strings[rec.index_key as usize..]);
            if name.is_empty() {
                continue;
            }

            self.nodes.insert(name, TableNode::new());
            // read_pascal_string(pstring: &[u8])
        }

        Ok(())
    }
}

struct DiskHeader {
    version: u16,
    sort_order: u16,
    time_created: u32,
    time_last_saved: u32,
    flags: u32,
}

impl DiskHeader {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        let version = u16::from_be_bytes(bytes[0..2].try_into()?);
        let sort_order = u16::from_be_bytes(bytes[2..4].try_into()?);
        let time_created = u32::from_be_bytes(bytes[4..8].try_into()?);
        let time_last_saved = u32::from_be_bytes(bytes[8..12].try_into()?);
        let flags = u32::from_be_bytes(bytes[12..16].try_into()?);

        Ok(Self {
            version,
            sort_order,
            time_created,
            time_last_saved,
            flags,
        })
    }
}

struct DiskSymbolRecord {
    index_key: u32,
    value_type: u8,
    version: u8,
    data: [u8; 4],
}

impl DiskSymbolRecord {
    pub fn new(bytes: &[u8]) -> Result<Self> {
        let index_key = u32::from_be_bytes(bytes[0..4].try_into()?);
        let value_type = bytes[4];
        let version = bytes[5];
        let mut data = [0; 4];
        data.clone_from_slice(&bytes[6..10]);

        Ok(Self {
            index_key,
            value_type,
            version,
            data,
        })
    }
}

fn split_buffer(buffer: &[u8]) -> Result<(&[u8], &[u8])> {
    let u32_size = std::mem::size_of::<u32>();
    let first_buffer_size = u32::from_be_bytes(buffer[0..u32_size].try_into()?) as usize;
    let index = u32_size;
    let first = &buffer[index..(index + first_buffer_size)];

    let index = index + first_buffer_size;
    let second_buffer_size = buffer.len() - (first_buffer_size + u32_size);
    let second = &buffer[index..(index + second_buffer_size)];

    Ok((first, second))
}
