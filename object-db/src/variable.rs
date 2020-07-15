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
use db::{DBAddress, Database};

pub trait LoadFromBytes {
    fn load_from_bytes(bytes: &[u8]) -> Result<Self>
    where
        Self: std::marker::Sized;
}

pub struct InMemoryValue<T: LoadFromBytes> {
    pub data: T,
    old_address: DBAddress,
}

pub enum VariableData<T: LoadFromBytes> {
    InMemory(InMemoryValue<T>),
    OnDisk(DBAddress),
}

pub struct Variable<'a, T: LoadFromBytes> {
    // id: u16,
    may_affect_display: bool,
    is_system_table: bool,
    pub data: VariableData<T>,
    db: &'a mut Database,
}

impl<'a, T: LoadFromBytes> Variable<'a, T> {
    pub fn new_on_disk(db: &'a mut Database, address: DBAddress) -> Self {
        Self {
            may_affect_display: false,
            is_system_table: false,
            data: VariableData::OnDisk(address),
            db,
        }
    }

    pub fn new_in_memory(db: &'a mut Database, data: T, old_address: DBAddress) -> Self {
        Self {
            may_affect_display: false,
            is_system_table: false,
            data: VariableData::InMemory(InMemoryValue { data, old_address }),
            db,
        }
    }

    pub fn load_from_disk(&mut self) -> Result<()> {
        match self.data {
            VariableData::OnDisk(address) => {
                let block = self.db.read_block(address)?;
                let data = T::load_from_bytes(&block)?;
                self.data = VariableData::InMemory(InMemoryValue {
                    data,
                    old_address: address,
                });
            }
            VariableData::InMemory(_) => {
                // nothing to do, it's already in memrory
            }
        };

        Ok(())
    }

    // pub fn load_from_disk(&mut self) -> Result<()> {
    //     match self.data {
    //         VariableData::OnDisk(address) => {
    //             let block = self.db.read_block(address)?;
    //             self.data = VariableData::InMemory(InMemoryValue {
    //                 data: block,
    //                 old_address: address,
    //             });
    //         }

    //         VariableData::InMemory(_) => {
    //             // nothing to do, it's already in memrory
    //         }
    //     };

    //     Ok(())
    // }
}

// impl<'a> Variable<'a, Vec<u8>> {
//     pub fn load_from_disk(&mut self) -> Result<()> {
//         match self.data {
//             VariableData::OnDisk(address) => {
//                 let block = self.db.read_block(address)?;
//                 let tbl = Table::load_from_bytes(&block);
//                 self.data = VariableData::InMemory(InMemoryValue {
//                     data: block,
//                     old_address: address,
//                 });
//             }

//             VariableData::InMemory(_) => {
//                 // nothing to do, it's already in memrory
//             }
//         };

//         Ok(())
//     }
// }
