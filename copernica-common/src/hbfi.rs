use {
    crate::{constants},
    anyhow::Result,
    borsh::{BorshDeserialize, BorshSerialize},
    sha3::{Digest, Sha3_512},
    std::fmt,
};

pub type BFI = [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]; // Bloom Filter Index

#[derive(Clone, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
// how to implement hierarchical routing...
// it should be done at node level
// if more than 1 link has an h3 then start route on h2
// if more than 2 links have h2 then route on h1... think about this for a while.
pub struct HBFI {
    // Hierarchical Bloom Filter Index
    //pub h3: BFI,  // level 3 hierarchy - most coarse
    //pub h2: BFI,  // level 2 hierarchy - comme ci, comme ça
    pub h1: BFI, // level 1 hierarchy - most fine
    pub id: BFI, // publisher id
    pub os: u64, // offset into h1 level of data
}

impl HBFI {
    pub fn new(h1: &str, id: &str) -> Result<HBFI> {
        Ok(HBFI {
            h1: bloom_filter_index(h1)?,
            id: bloom_filter_index(id)?,
            os: 0,
        })
    }

    #[cfg(test)]
    pub fn new_test(h1: BFI, id: BFI, os: u64) -> Self {
        HBFI { h1, id, os }
    }
    pub fn to_vec(&self) -> Vec<BFI> {
        vec![self.id.clone(), self.h1.clone()]
    }
    pub fn offset(mut self, os: u64) -> Self {
        self.os = os;
        self
    }
}

impl fmt::Debug for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { h1, id, os } => write!(f, "{:?}::{:?}::{:?}", h1, id, os),
        }
    }
}

impl fmt::Display for HBFI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &*self {
            HBFI { h1, id, os } => write!(f, "{:?}::{:?}::{:?}", h1, id, os),
        }
    }
}

fn bloom_filter_index(
    s: &str,
) -> Result<[u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize]> {
    use std::str;
    let mut hasher = Sha3_512::new();
    hasher.input(s.as_bytes());
    let hash = hasher.result();
    let mut bloom_filter_index_array: BFI =
        [0; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
    let mut count = 0;
    for n in 0..constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH {
        let mut hasher = Sha3_512::new();
        hasher.input(format!("{:x}{}", hash, n));
        let hs = format!("{:x}", hasher.result());
        let subs = hs
            .as_bytes()
            .chunks(16)
            .map(str::from_utf8)
            .collect::<Result<Vec<&str>, _>>()?;
        let mut index: u64 = 0;
        for sub in subs {
            let o = u64::from_str_radix(&sub, 16)?;
            index = (index + o) % constants::BLOOM_FILTER_LENGTH;
        }
        bloom_filter_index_array[count] = index as u16;
        count += 1;
    }
    Ok(bloom_filter_index_array)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        packets::{Data, NarrowWaistPacket, LinkPacket},
        link::{ReplyTo},
    };

    #[test]
    fn test_bloom_filter_index() {
        let actual = bloom_filter_index("9".into()).unwrap();
        let expected: [u16; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize] =
            [4804, 63297, 3290, 20147];
        assert_eq!(actual, expected);
    }

    #[test]
    fn less_than_1472_bytes() {
        // https://gafferongames.com/post/packet_fragmentation_and_reassembly
        let h1: BFI = [u16::MAX; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
        let id: BFI = [u16::MAX; constants::BLOOM_FILTER_INDEX_ELEMENT_LENGTH as usize];
        let hbfi = HBFI::new_test(h1, id, u64::MAX);
        let data = [0; constants::FRAGMENT_SIZE as usize];
        let data: Data = Data { len: constants::FRAGMENT_SIZE, data};
        let nw: NarrowWaistPacket = NarrowWaistPacket::Response { hbfi, data, offset: u64::MAX, total: u64::MAX };
        let reply_to: ReplyTo = ReplyTo::UdpIp("127.0.0.1:50000".parse().unwrap());
        let wp: LinkPacket = LinkPacket { reply_to, nw };
        let wp_ser = wp.try_to_vec().unwrap();
        let lt1472 = if wp_ser.len() <= 1472 { true } else { false };
        assert_eq!(true, lt1472);
    }
}
