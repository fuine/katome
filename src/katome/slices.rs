//! Representation of string for vertex in De Bruijn Graph.

use asm::SEQUENCES;
use compress::{decompress_edge, decompress_last_char_edge, decompress_node, extend_edge};
use prelude::{COMPRESSED_K1_SIZE, K1_SIZE, Idx};

use std::cmp;
use std::fmt;
use std::hash;
use std::option::Option;
use std::str;

#[derive(Copy, Clone, Default, Debug)]
/// View of the compressed edge.
pub struct EdgeSlice {
    offset: Idx,
}

impl EdgeSlice {
    pub fn merge(&self, other: EdgeSlice) {
        let self_idx = self.idx();
        let other_idx = other.idx();
        let mut s = SEQUENCES.write();
        let other_uncompressed = decompress_edge(&*s[other_idx]);
        let tmp = extend_edge(&*s[self_idx], &other_uncompressed[unsafe { K1_SIZE }..]);
        // clear other as we won't use it anymore
        s[other_idx] = Vec::new().into_boxed_slice();
        // swap to the new value
        s[self_idx] = tmp.into_boxed_slice();
    }

    pub fn remainder(&self) -> String {
        let mut name = self.name();
        name.drain(..unsafe { K1_SIZE });
        name
    }
}

#[derive(Copy, Clone, Default, Debug)]
/// View of the compressed node (string of length `K1_SIZE`).
pub struct NodeSlice {
    offset: Idx,
}

impl From<NodeSlice> for EdgeSlice {
    fn from(from: NodeSlice) -> EdgeSlice {
        EdgeSlice::new(from.idx())
    }
}

/// Wrapper around slice of read (`String`).
/// Works on the global, static `Sequences` representing all reads.
///
/// It stores information about offset and assumes k-mer size of `K_SIZE`
pub trait BasicSlice {
    /// Creates associated value with the given offset.
    fn new(offset: Idx) -> Self where Self: Sized;

    /// Gets `String` representation of the slice.
    fn name(&self) -> String;

    /// Gets bytestring representation of the slice.
    fn byte_name(&self) -> Vec<u8>;

    /// Gets last `char` of the slice.
    fn last_char(&self) -> char;

    fn idx(&self) -> usize;
    fn offset(&self) -> usize;
}

macro_rules! add_impls {
    ($($t:tt),*) => ($(
        impl fmt::Display for $t {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "{}", self.idx())
            }
        }

        impl hash::Hash for $t {
            fn hash<H>(&self, state: &mut H) where H: hash::Hasher {
                let s = SEQUENCES.read();
                let slice = get_slice!($t, self, s);
                slice.hash(state)
            }
        }

        impl cmp::PartialEq for $t {
            fn eq(&self, other: &Self) -> bool {
                let s = SEQUENCES.read();
                let slice_ = get_slice!($t, self, s);
                let slice_oth = get_slice!($t, other, s);
                slice_ == slice_oth
            }
        }

        impl cmp::PartialOrd for $t {
            fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
                let s = SEQUENCES.read();
                let slice_ = get_slice!($t, self, s);
                let slice_oth = get_slice!($t, other, s);
                slice_.partial_cmp(slice_oth)
            }
        }

        impl cmp::Eq for $t { }

        impl cmp::Ord for $t {
            fn cmp(&self, other: &Self) -> cmp::Ordering {
                let s = SEQUENCES.read();
                let slice_ = get_slice!($t, self, s);
                let slice_oth = get_slice!($t, other, s);
                slice_.cmp(slice_oth)
            }
        }
    )*)
}

macro_rules! get_slice {
    (EdgeSlice, $i:ident, $s:ident) => {{ get_slice_edge!($i, $s) }};
    (NodeSlice, $i:ident, $s:ident) => {{ get_slice_node!($i, $s) }};
}

macro_rules! get_slice_node {
    ($i:ident, $s:ident) => {{
        let node_offset = $i.offset() % 2;
        let s = unsafe{ COMPRESSED_K1_SIZE } * node_offset;
        let t = unsafe{ COMPRESSED_K1_SIZE } + s;
        &$s[$i.idx()][s..t]
    }};
}

macro_rules! get_slice_edge {
    ($i:ident, $s: ident) => {{
        &*$s[$i.idx()]
    }};
}
add_impls!(EdgeSlice, NodeSlice);

impl BasicSlice for EdgeSlice {
    fn new(offset_: Idx) -> EdgeSlice {
        EdgeSlice { offset: offset_ }
    }

    fn name(&self) -> String {
        let s = SEQUENCES.read();
        unsafe { String::from_utf8_unchecked(decompress_edge(get_slice_edge!(self, s))) }
    }

    fn byte_name(&self) -> Vec<u8> {
        let s = SEQUENCES.read();
        Vec::from(decompress_edge(get_slice_edge!(self, s)))
    }

    fn last_char(&self) -> char {
        let s = SEQUENCES.read();
        decompress_last_char_edge(get_slice_edge!(self, s))
    }

    fn idx(&self) -> usize {
        self.offset as usize
    }

    fn offset(&self) -> usize {
        self.offset as usize
    }
}

impl BasicSlice for NodeSlice {
    fn new(offset_: Idx) -> NodeSlice {
        NodeSlice { offset: offset_ }
    }

    fn name(&self) -> String {
        let s = SEQUENCES.read();
        let slice = get_slice_node!(self, s);
        unsafe { String::from_utf8_unchecked(decompress_node(slice)) }
    }

    fn byte_name(&self) -> Vec<u8> {
        let s = SEQUENCES.read();
        Vec::from(decompress_node(get_slice_node!(self, s)))
    }

    fn last_char(&self) -> char {
        let s = SEQUENCES.read();
        if self.offset() % 2 > 0 {
            s[self.idx()][s.len() - 1] as char
        }
        else {
            s[self.idx()][0] as char
        }
    }

    fn idx(&self) -> usize {
        (self.offset / 2) as usize
    }

    fn offset(&self) -> usize {
        self.offset as usize
    }
}

#[cfg(test)]
mod tests {
    extern crate rand;
    pub use ::asm::SEQUENCES;
    pub use ::asm::lock::LOCK;
    pub use ::compress::{compress_kmer, kmer_to_edge};
    pub use ::prelude::{K1_SIZE, K_SIZE};
    pub use self::rand::Rng;
    pub use self::rand::thread_rng;
    pub use std::collections::hash_map::DefaultHasher;
    pub use std::hash::Hash;
    pub use super::*;
    pub use std::panic::catch_unwind;

    macro_rules! setup {
        ($l:ident, $n:ident) => {
            // global lock on sequences for test
            let $l = LOCK.lock().unwrap();
            // initialize with random data
            let $n = thread_rng()
                .gen_iter::<u8>()
                .take(unsafe{K_SIZE})
                .map(|x| {
                    match x % 4 {
                        0 => 65u8,
                        1 => 67u8,
                        2 => 84u8,
                        3 => 71u8,
                        _ => unreachable!(),
                    }
                })
            .collect::<Vec<u8>>();
            {
                let mut seq = SEQUENCES.write();
                seq.clear();
                let mut shifted_kmer = Vec::from(&$n[1..]);
                shifted_kmer.push(b'A');
                let compressed_kmer = compress_kmer(&$n);
                let compressed_shifted_kmer = compress_kmer(&shifted_kmer);
                let compressed_edge = kmer_to_edge(&compressed_kmer);
                seq.push(compressed_kmer.clone().into_boxed_slice());
                seq.push(compressed_kmer.clone().into_boxed_slice());
                seq.push(compressed_shifted_kmer.clone().into_boxed_slice());
                seq.push(compressed_edge.clone().into_boxed_slice());
                seq.push(compressed_edge.clone().into_boxed_slice());
            }
        }
    }
    macro_rules! test {
        ($n:ident, $b:block) => {
            #[test]
            fn $n() {
                let result = {
                    $b
                };
                assert!(result.is_ok());
            }
        }
    }
    mod ns {
        use super::*;

        test!(creates_new, {
            setup!(_l, name);
            catch_unwind(|| {
                let ns = NodeSlice::new(0);
                let st = unsafe { String::from_utf8_unchecked(name[..K1_SIZE].to_vec()) };
                assert_eq!(ns.name(), st);
                let ns = NodeSlice::new(1);
                assert_eq!(ns.name(),
                           unsafe { String::from_utf8_unchecked(name[1..1 + K1_SIZE].to_vec()) });
            })
        });

        test!(compares_similar, {
            setup!(_l, name);
            catch_unwind(|| {
                let ns1 = NodeSlice::new(0);
                let ns2 = NodeSlice::new(2);
                assert_eq!(ns1, ns2);
            })
        });

        test!(compares_shifted_names, {
            setup!(_l, name);
            catch_unwind(|| {
                let ns1 = NodeSlice::new(1);
                let ns2 = NodeSlice::new(4);
                assert_eq!(ns1.name(), ns2.name());
            })
        });

        test!(compares_shifted_slices, {
            setup!(_l, name);
            catch_unwind(|| {
                let ns1 = NodeSlice::new(1);
                let ns2 = NodeSlice::new(4);
                assert_eq!(ns1, ns2);
            })
        });

        test!(compares_hashes, {
            setup!(_l, name);
            catch_unwind(|| {
                let ns1 = NodeSlice::new(0);
                let ns2 = NodeSlice::new(2);
                let mut hasher = DefaultHasher::new();
                assert!(ns1.hash(&mut hasher) == ns2.hash(&mut hasher));
            })
        });
    }

    mod es {
        use super::*;

        test!(creates_new, {
            setup!(_l, name);
            catch_unwind(|| {
                let es = EdgeSlice::new(3);
                assert_eq!(es.name(), unsafe { String::from_utf8_unchecked(name) });
            })
        });

        test!(compares_similar, {
            setup!(_l, name);
            catch_unwind(|| {
                let es1 = EdgeSlice::new(3);
                let es2 = EdgeSlice::new(4);
                assert_eq!(es1, es2);
            })
        });

        test!(compares_hashes, {
            setup!(_l, name);
            catch_unwind(|| {
                let es1 = EdgeSlice::new(3);
                let es2 = EdgeSlice::new(4);
                let mut hasher = DefaultHasher::new();
                assert!(es1.hash(&mut hasher) == es2.hash(&mut hasher));
            })
        });
    }
}
