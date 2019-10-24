#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate sha3;

#[cfg(test)]
extern crate tar;
#[cfg(test)]
extern crate flate2;
#[cfg(test)]
extern crate bitvec;

mod index;

use crate::{index::generate_sdr_index};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Packet {
    Interest { sdri: Vec<Vec<u16>> },
    Data     { sdri: Vec<Vec<u16>>, data: Vec<u8> },
}

pub fn mk_interest(name: String) -> Packet {
    Packet::Interest {
        sdri: generate_sdr_index(name)
        // more to come
    }
}

pub fn mk_data(name: String, data: Vec<u8>) -> Packet {
    Packet::Data {
        sdri: generate_sdr_index(name),
        data: data,
    }
}
