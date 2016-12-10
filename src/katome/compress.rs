//! Efficient compression/decompression algorithm for sequences with basic
//! nucleotydes.

use prelude::{K1_SIZE, K_SIZE, COMPRESSED_K1_SIZE};

/// Number of characters fitting inside the byte.
pub const CHARS_PER_BYTE: usize = 4;

/// Compress kmer representation. Output is roughly 2 times smaller,
/// because it needs to store two compressed nodes independently.
///
/// Symbols are put into chunks of 4 and represented as single byte.
pub fn compress_kmer(kmer: &[u8]) -> Vec<u8> {
    assert!(kmer.len() > 2);
    let compressed_size = 2 * ((kmer.len() as f64 - 1.0) / 4.0).ceil() as usize;
    let mut compressed = Vec::with_capacity(compressed_size);
    let start_node = &kmer[..kmer.len() - 1];
    let end_node = &kmer[1..];
    compress_node(start_node, &mut compressed);
    compress_node(end_node, &mut compressed);
    compressed
}

/// Compress node representation. Output is roughly 4 times smaller,
/// uses 2 bits per symbol.
///
/// Supports only A, C, T, G as it's alphabet.
pub fn compress_node(slice: &[u8], collection: &mut Vec<u8>) {
    let mut byte = 0u8;
    for chunk in slice.chunks(CHARS_PER_BYTE) {
        for c in chunk {
            byte = encode_fasta_symbol(*c, byte);
        }
        collection.push(byte);
        byte = 0u8;
    }
    if let Some(x) = slice.chunks(CHARS_PER_BYTE).last() {
        let l = CHARS_PER_BYTE - x.len();
        // move remaining bytes to the end of the u8, such that they can be
        // properly decrompressed
        if l != 0 {
            let len = collection.len();
            collection[len - 1] <<= 2 * l;
        }
    }
}

/// Decompress node representation from the compressed form. Returns vector of
/// symbols.
pub fn decompress_node(node: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(unsafe { K1_SIZE });
    for chunk in node {
        output.extend(decode_compressed_chunk(*chunk).iter().cloned());
    }
    output.truncate(unsafe { K1_SIZE });
    output
}

/// Decompress kmer representation from the compressed form. Returns vector of
/// symbols.
pub fn decompress_kmer(kmer: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(unsafe { K_SIZE });
    let slice_ = &kmer[..unsafe { COMPRESSED_K1_SIZE }];
    let dec = decompress_node(slice_);
    output.extend_from_slice(&dec);
    output.push(get_last_char_from_node(&kmer[unsafe { COMPRESSED_K1_SIZE }..]));
    output
}

fn get_last_char_from_node(node: &[u8]) -> u8 {
    let padding = unsafe { K1_SIZE } % CHARS_PER_BYTE;
    let last_byte: u8 = node[node.len() - 1];
    let padding = (CHARS_PER_BYTE - padding) % CHARS_PER_BYTE;
    decompress_char(last_byte, padding) as u8
}

pub fn change_char_in_chunk(mut chunk: u8, padding: usize, to: u8) -> u8 {
    let mask = 0b11111100 << (2 * padding);
    let compressed_char = encode_fasta_symbol(to, 0u8) << (2 * padding);
    chunk &= mask;
    chunk |= compressed_char;
    chunk
}

/// Change last character in the compressed edge representation.
pub fn change_last_char_in_edge(edge: &[u8], to: u8) -> Vec<u8> {
    let mut output = edge.to_vec();
    let padding = output[0] as usize;
    // mask for zeroing out the last char
    let len = output.len() - 1;
    let last_byte = change_char_in_chunk(output[len], padding, to);
    output[len] = last_byte;
    output
}

pub fn add_char_to_edge(edge: &[u8], mut chr: u8) -> Vec<u8> {
    assert!(edge.len() > 1);
    let padding = edge[0];
    let len = edge.len() - 1;
    let new_pad = padding.wrapping_sub(1) % CHARS_PER_BYTE as u8;
    let mask = 0b11111100 << (2 * new_pad);
    chr = encode_fasta_symbol(chr, 0u8);
    if new_pad != 3 {
        let mut output = Vec::from(edge);
        output[len] &= mask;
        output[len] |= chr << (2 * new_pad);
        output[0] = new_pad;
        output
    }
    else {
        let mut output = Vec::with_capacity(len + 1);
        output.extend_from_slice(edge);
        // we need to create new block
        output[0] = new_pad;
        output.push(chr << (2 * new_pad));
        output
    }
}

/// Recompress kmer as edge representation.
pub fn kmer_to_edge(kmer: &[u8]) -> Vec<u8> {
    compress_edge(&decompress_kmer(kmer))
}

/// Compress edge (string with length > 2) representation. Output is roughly
/// 4 times smaller, uses 2 bits per symbol + 1 byte to denote padding.
///
/// Symbols are put into chunks of 4 and represented as single byte. First byte
/// denotes padding used in the last byte. Consecutive characters will take 2
/// bits starting from the most significant bits.
///
/// # Example
///
/// ```
/// use katome::compress::compress_edge;
/// let compressed = compress_edge(b"AGGTCG");
/// assert_eq!(vec![2u8, 0b00101011, 0b01100000], compressed);
/// ```
pub fn compress_edge(edge: &[u8]) -> Vec<u8> {
    assert!(edge.len() > 2);
    let compressed_size = 1 + ((edge.len() as f64) / 4.0).ceil() as usize;
    let mut compressed = Vec::with_capacity(compressed_size);
    let mut byte = 0u8;
    compressed.push(0u8);
    // CHARS_PER_BYTE characters per byte
    for chunk in edge.chunks(CHARS_PER_BYTE) {
        for c in chunk {
            byte = encode_fasta_symbol(*c, byte);
        }
        compressed.push(byte);
        byte = 0u8;
    }
    let padding = CHARS_PER_BYTE -
                  edge.chunks(CHARS_PER_BYTE).last().expect("This should never fail").len();
    // move remaining bytes to the end of the u8, such that they can be
    // properly decrompressed
    compressed[compressed_size - 1] <<= 2 * padding;
    compressed[0] = padding as u8;
    compressed
}

/// Decompress edge representation from the compressed form. Returns vector of symbols.
///
/// # Example
///
/// ```
/// use katome::compress::decompress_edge;
/// let decompressed = decompress_edge(&vec![2u8, 0b00101011, 0b01100000]);
/// assert_eq!(b"AGGTCG", decompressed.as_slice());
/// ```
pub fn decompress_edge(edge: &[u8]) -> Vec<u8> {
    let padding = edge[0] as usize;
    let mut output: Vec<u8> = Vec::with_capacity(((edge.len() - 1) * CHARS_PER_BYTE) - padding);
    let slice_ = &edge[1..];
    for chunk in slice_ {
        output.extend(decode_compressed_chunk(*chunk).iter().cloned());
    }
    let len = output.len() - padding;
    output.truncate(len);
    output
}

pub fn decompress_last_char_edge(edge: &[u8]) -> char {
    let padding = edge[0] as usize;
    decompress_char(edge[edge.len() - 1], padding)
}

#[inline]
/// Decompress single character from chunk.
pub fn decompress_char(mut chunk: u8, padding: usize) -> char {
    let mask = 3u8;
    chunk >>= 2 * padding;
    match chunk & mask {
        0 => 'A', // A
        1 => 'C', // C
        2 => 'G', // G
        3 => 'T', // T
        _ => unreachable!(),
    }
}

#[inline]
/// Compress single character and put it inside carrier.
pub fn encode_fasta_symbol(symbol: u8, carrier: u8) -> u8 {
    // make room for a new character
    let x = carrier << 2;
    match symbol {
        b'A' => x,
        b'C' => x | 1,
        b'G' => x | 2,
        b'T' => x | 3,
        u => panic!("Unknown FASTA character found: {}", u),
    }
}

#[inline]
fn decode_compressed_chunk(mut chunk: u8) -> [u8; CHARS_PER_BYTE] {
    let mut output = [0u8; CHARS_PER_BYTE];
    // two first bits
    let mask = 3u8;
    for i in (0..CHARS_PER_BYTE).rev() {
        let symbol: u8 = chunk & mask;
        chunk >>= 2;
        output[i] = match symbol {
            0 => b'A',
            1 => b'C',
            2 => b'G',
            3 => b'T',
            _ => unreachable!(),
        };
    }
    output
}

#[cfg(test)]
mod tests {
    extern crate rand;
    use prelude::K_SIZE;
    use self::rand::Rng;
    use self::rand::thread_rng;
    use super::*;
    use super::decode_compressed_chunk;

    #[test]
    fn properly_compresses_single_chunk() {
        let a = "ACTG";
        let proper_result = a.bytes().collect::<Vec<_>>();
        let mut byte = 0u8;
        for c in a.bytes() {
            byte = encode_fasta_symbol(c, byte);
        }
        let decoded = decode_compressed_chunk(byte);
        assert_eq!(proper_result, decoded);
    }

    #[test]
    fn properly_decompresses_last_char() {
        let a = "ACTG";
        let mut byte = 0u8;
        for c in a.bytes() {
            byte = encode_fasta_symbol(c, byte);
        }
        let decoded = decompress_char(byte, 0);
        assert_eq!('G', decoded);
    }

    #[test]
    fn properly_compresses_vertex() {
        let name = thread_rng()
            .gen_iter::<u8>()
            .take(unsafe { K_SIZE })
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
        let compressed = compress_kmer(name.as_slice());
        let decoded = decompress_kmer(compressed.as_slice());
        assert_eq!(name, decoded);
    }

    #[test]
    fn adds_char_to_edge() {
        // pad 0
        let compressed = compress_edge(b"AGGT");
        assert_eq!(vec![0u8, 0b00101011], compressed);
        let added = add_char_to_edge(&compressed, b'T');
        assert_eq!(vec![3u8, 0b00101011, 0b11000000], added);

        // pad 1
        let compressed = compress_edge(b"AGGTCGG");
        assert_eq!(vec![1u8, 0b00101011, 0b01101000], compressed);
        let added = add_char_to_edge(&compressed, b'T');
        assert_eq!(vec![0u8, 0b00101011, 0b01101011], added);

        // pad 2
        let compressed = compress_edge(b"AGGTCG");
        assert_eq!(vec![2u8, 0b00101011, 0b01100000], compressed);
        let added = add_char_to_edge(&compressed, b'T');
        assert_eq!(vec![1u8, 0b00101011, 0b01101100], added);

        // pad 3
        let compressed = compress_edge(b"AGGTC");
        assert_eq!(vec![3u8, 0b00101011, 0b01000000], compressed);
        let added = add_char_to_edge(&compressed, b'G');
        assert_eq!(vec![2u8, 0b00101011, 0b01100000], added);
    }

    #[test]
    fn change_last_char_edge() {
        let in_ = vec![3u8, 0b00101011, 0b11000000];
        let out = change_last_char_in_edge(&in_, b'A');
        assert_eq!(vec![3u8, 0b00101011, 0b00000000], out);
    }
}
