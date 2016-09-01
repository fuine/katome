use data::primitives::{K_SIZE, Idx};
use asm::assembler::SEQUENCES;
use std::cmp;
use std::hash;
use std::str;
use std::option::Option;

/// Wrapper around slice of read.
/// Works on the global, static `Vec<u8>`.
#[derive(Eq, Clone, Default, Debug)]
pub struct ReadSlice {
    pub offset: Idx,
}

impl ReadSlice {
    /// Create new slice with the given offset.
    pub fn new(offset_: Idx) -> ReadSlice {
        ReadSlice { offset: offset_ }
    }

    /// Get `String` representation of the slice.
    pub fn name(&self) -> String {
        unwrap!(str::from_utf8(&unwrap!(SEQUENCES.read())[self.offset as usize..(self.offset+K_SIZE) as usize])).to_string()
    }

    /// Get last `char` of the slice.
    pub fn last_char(&self) -> char {
        unwrap!(SEQUENCES.read())[(self.offset + K_SIZE - 1) as usize] as char
    }
}

impl hash::Hash for ReadSlice {
    fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
        let slice_ = &unwrap!(SEQUENCES.read())[self.offset as usize..(self.offset + K_SIZE) as usize];
        slice_.hash(state)
    }
}

impl cmp::PartialEq for ReadSlice {
    fn eq(&self, other: &ReadSlice) -> bool {
        let slice_ = &unwrap!(SEQUENCES.read())[self.offset as usize..(self.offset + K_SIZE) as usize];
        let other_slice_ =
            &unwrap!(SEQUENCES.read())[other.offset as usize..(other.offset + K_SIZE) as usize];
        slice_ == other_slice_
    }
}

impl cmp::PartialOrd for ReadSlice {
    fn partial_cmp(&self, other: &ReadSlice) -> Option<cmp::Ordering> {
        let slice_ = &unwrap!(SEQUENCES.read())[self.offset as usize..(self.offset + K_SIZE) as usize];
        let other_slice_ =
            &unwrap!(SEQUENCES.read())[other.offset as usize..(other.offset + K_SIZE) as usize];
        slice_.partial_cmp(other_slice_)
    }
}

impl cmp::Ord for ReadSlice {
    fn cmp(&self, other: &ReadSlice) -> cmp::Ordering {
        let slice_ = &unwrap!(SEQUENCES.read())[self.offset as usize..(self.offset + K_SIZE) as usize];
        let other_slice_ =
            &unwrap!(SEQUENCES.read())[other.offset as usize..(other.offset + K_SIZE) as usize];
        slice_.cmp(other_slice_)
    }
}

#[macro_export]
macro_rules! RS {
    ($o:expr) => (ReadSlice::new($o));
}


#[cfg(test)]
mod tests {
    extern crate rand;
    pub use super::*;
    pub use self::rand::Rng;
    pub use self::rand::thread_rng;
    pub use ::data::primitives::K_SIZE;
    pub use ::asm::assembler::SEQUENCES;
    pub use std::hash::SipHasher;
    pub use std::hash::Hash;
    pub use ::asm::assembler::lock::LOCK;

    describe! rs {
        before_each {
            // global lock on sequences for test
            let _l = LOCK.lock().unwrap();
            // initialize with random data
            let name = thread_rng()
                .gen_ascii_chars()
                .take(K_SIZE)
                .collect::<String>();
            {
                let mut seq = SEQUENCES.write().unwrap();
                seq.clear();
                seq.extend(name.clone().into_bytes());
                seq.extend(name.clone().into_bytes());
            }
        }

        it "creates new RS" {
            let rs = ReadSlice::new(0);
            assert_eq!(rs.name(), name);
        }

        it "creates new RS with macro" {
            let rs = RS!(0);
            assert_eq!(rs.name(), name);
        }

        it "compares similar RSes" {
            let rs1 = RS!(0);
            let rs2 = RS!(K_SIZE);
            assert_eq!(rs1, rs2);
        }

        it "compares hashes" {
            let rs1 = RS!(0);
            let rs2 = RS!(K_SIZE);
            let mut hasher = SipHasher::new();
            assert!(rs1.hash(&mut hasher) == rs2.hash(&mut hasher));
        }
    }
}
