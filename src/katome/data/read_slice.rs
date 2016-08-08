use data::types::{VertexId, K_SIZE};
use asm::assembler::{SEQUENCES};
use std::cmp;
use std::hash;
use std::str;
use std::option::{Option};

#[derive(Eq, Clone)]
pub struct ReadSlice {
    pub offset: VertexId,
}

impl ReadSlice {
    pub fn new(offset_: VertexId) -> ReadSlice {
        ReadSlice {
            offset: offset_
        }
    }

    pub fn name(&self) -> String {
        str::from_utf8(&SEQUENCES.read().unwrap()[self.offset as usize..(self.offset+K_SIZE) as usize]).unwrap().to_string()
    }

    pub fn last_char(&self) -> char {
        SEQUENCES.read().unwrap()[(self.offset + K_SIZE - 1) as usize] as char
    }

    pub fn get_slice(&self) -> Vec<u8> {
        SEQUENCES.read().unwrap()[self.offset as usize..(self.offset+K_SIZE) as usize].to_vec().clone()
    }
}

impl hash::Hash for ReadSlice {
    fn hash<H>(&self, state: &mut H)
            where H: hash::Hasher {
        self.name().hash(state)
    }
}

impl cmp::PartialEq for ReadSlice {
    fn eq(&self, other: &ReadSlice) -> bool{
        self.name() == other.name()
    }
}

impl cmp::PartialOrd for ReadSlice {
    fn partial_cmp(&self, other: &ReadSlice) -> Option<cmp::Ordering>{
        self.name().partial_cmp(&other.name())
    }
}

impl cmp::Ord for ReadSlice {
    fn cmp(&self, other: &ReadSlice) -> cmp::Ordering{
        self.name().cmp(&other.name())
    }
}

#[macro_export]
macro_rules! RS {
    ($o:expr) => (ReadSlice::new($o));
}


#[cfg(test)]
mod tests {
    use super::*;
    use ::data::types::{VecArc, K_SIZE};
    use ::asm::assembler::{SEQUENCES};
    use std::sync::Arc;
    use std::cell::RefCell;
    use std::iter::repeat;
    use std::hash::SipHasher;
    use std::hash::Hash;

    #[test]
    fn new_read_slice() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // initialize with random data
        let mut name: String = repeat("very_long_and_complicated_name").take(200).collect::<String>();
        name.truncate(K_SIZE);
        {
            SEQUENCES.write().unwrap().extend(name.clone().into_bytes());
        }
        let rs = ReadSlice::new(0);
        assert_eq!(rs.name(), name);
    }

    // #[test]
    // fn new_read_slice_from_macro() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // // initialize with random data
        // let mut name: String = repeat("very_long_and_complicated_name").take(200).collect::<String>();
        // name.truncate(K_SIZE);
        // sequences.borrow_mut().extend(name.clone().into_bytes());
        // let rs = RS!(sequences, 0);
        // assert_eq!(rs.name(), name);
    // }

    // #[test]
    // fn get_slice() {
        // let sequences: VecArc = Arc::new(RefCell::new(Vec::new()));
        // // initialize with random data
        // let mut name: String = repeat("very_long_and_complicated_name").take(200).collect::<String>();
        // name.truncate(K_SIZE);
        // sequences.borrow_mut().extend(name.clone().into_bytes());
        // let rs = ReadSlice::new(sequences.clone(), 0);
        // assert_eq!(rs.get_slice(), name.into_bytes());
    // }

    // #[test]
    // fn compare_similar_read_slices() {
        // let sequences1: VecArc = Arc::new(RefCell::new(Vec::new()));
        // let sequences2: VecArc = Arc::new(RefCell::new(Vec::new()));
        // // initialize with random data
        // let mut name: String = repeat("very_long_and_complicated_name").take(200).collect::<String>();
        // name.truncate(K_SIZE);
        // sequences1.borrow_mut().extend(name.clone().into_bytes());
        // sequences2.borrow_mut().extend(name.clone().into_bytes());
        // let rs1 = RS!(sequences1, 0);
        // let rs2 = RS!(sequences2, 0);
        // assert!(rs1 == rs2);
    // }

    // #[test]
    // fn compare_hashes() {
        // let sequences1: VecArc = Arc::new(RefCell::new(Vec::new()));
        // let sequences2: VecArc = Arc::new(RefCell::new(Vec::new()));
        // // initialize with random data
        // let mut name: String = repeat("very_long_and_complicated_name").take(200).collect::<String>();
        // name.truncate(K_SIZE);
        // sequences1.borrow_mut().extend(name.clone().into_bytes());
        // sequences2.borrow_mut().extend(name.clone().into_bytes());
        // let rs1 = RS!(sequences1, 0);
        // let rs2 = RS!(sequences2, 0);
        // let mut hasher = SipHasher::new();
        // assert!(rs1.hash(&mut hasher) == rs2.hash(&mut hasher));
    // }
}
