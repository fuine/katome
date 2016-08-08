use data::types::{K_SIZE, VertexId};
use asm::assembler::SEQUENCES;
use std::cmp;
use std::hash;
use std::str;
use std::option::Option;

/// Wrapper around slice of read.
/// Works on the global, static `Vec<u8>`.
#[derive(Eq, Clone)]
pub struct ReadSlice {
    pub offset: VertexId,
}

impl ReadSlice {
    /// Create new slice with the given offset.
    pub fn new(offset_: VertexId) -> ReadSlice {
        ReadSlice { offset: offset_ }
    }

    /// Get `String` representation of the slice.
    pub fn name(&self) -> String {
        str::from_utf8(&SEQUENCES.read().unwrap()[self.offset as usize..(self.offset+K_SIZE) as usize]).unwrap().to_string()
    }

    /// Get last `char` of the slice.
    pub fn last_char(&self) -> char {
        SEQUENCES.read().unwrap()[(self.offset + K_SIZE - 1) as usize] as char
    }
}

impl hash::Hash for ReadSlice {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        let slice_ = &SEQUENCES.read()
            .unwrap()[self.offset as usize..(self.offset + K_SIZE) as usize];
        slice_.hash(state)
    }
}

impl cmp::PartialEq for ReadSlice {
    fn eq(&self, other: &ReadSlice) -> bool {
        let slice_ = &SEQUENCES.read()
            .unwrap()[self.offset as usize..(self.offset + K_SIZE) as usize];
        let other_slice_ =
            &SEQUENCES.read().unwrap()[other.offset as usize..(other.offset + K_SIZE) as usize];
        slice_ == other_slice_
    }
}

impl cmp::PartialOrd for ReadSlice {
    fn partial_cmp(&self, other: &ReadSlice) -> Option<cmp::Ordering> {
        let slice_ = &SEQUENCES.read()
            .unwrap()[self.offset as usize..(self.offset + K_SIZE) as usize];
        let other_slice_ =
            &SEQUENCES.read().unwrap()[other.offset as usize..(other.offset + K_SIZE) as usize];
        slice_.partial_cmp(&other_slice_)
    }
}

impl cmp::Ord for ReadSlice {
    fn cmp(&self, other: &ReadSlice) -> cmp::Ordering {
        let slice_ = &SEQUENCES.read()
            .unwrap()[self.offset as usize..(self.offset + K_SIZE) as usize];
        let other_slice_ =
            &SEQUENCES.read().unwrap()[other.offset as usize..(other.offset + K_SIZE) as usize];
        slice_.cmp(&other_slice_)
    }
}

#[macro_export]
macro_rules! RS {
    ($o:expr) => (ReadSlice::new($o));
}


#[cfg(test)]
mod tests {
    extern crate rand;
    use rand::Rng;
    use super::*;
    use ::data::types::K_SIZE;
    use ::asm::assembler::SEQUENCES;
    use std::hash::SipHasher;
    use std::hash::Hash;

    #[test]
    fn new_read_slice() {
        // initialize with random data
        let name = rand::thread_rng()
            .gen_ascii_chars()
            .take(K_SIZE)
            .collect::<String>();
        {
            let mut seq = SEQUENCES.write().unwrap();
            seq.clear();
            seq.extend(name.clone().into_bytes());
        }
        let rs = ReadSlice::new(0);
        assert_eq!(rs.name(), name);
    }

    #[test]
    fn new_read_slice_from_macro() {
        // initialize with random data
        let name = rand::thread_rng()
            .gen_ascii_chars()
            .take(K_SIZE)
            .collect::<String>();
        {
            let mut seq = SEQUENCES.write().unwrap();
            seq.clear();
            seq.extend(name.clone().into_bytes());
        }
        let rs = RS!(0);
        assert_eq!(rs.name(), name);
    }

    #[test]
    fn compare_similar_read_slices() {
        // initialize with random data
        let name = rand::thread_rng()
            .gen_ascii_chars()
            .take(K_SIZE)
            .collect::<String>();
        {
            let mut seq = SEQUENCES.write().unwrap();
            seq.clear();
            seq.extend(name.clone().into_bytes());
            seq.extend(name.clone().into_bytes());
        }
        let rs1 = RS!(0);
        let rs2 = RS!(K_SIZE);
        assert!(rs1 == rs2);
    }

    #[test]
    fn compare_hashes() {
        // initialize with random data
        let name = rand::thread_rng()
            .gen_ascii_chars()
            .take(K_SIZE)
            .collect::<String>();
        {
            let mut seq = SEQUENCES.write().unwrap();
            seq.clear();
            seq.extend(name.clone().into_bytes());
            seq.extend(name.clone().into_bytes());
        }
        let rs1 = RS!(0);
        let rs2 = RS!(K_SIZE);
        let mut hasher = SipHasher::new();
        assert!(rs1.hash(&mut hasher) == rs2.hash(&mut hasher));
    }
}
