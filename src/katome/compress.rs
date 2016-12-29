//! Efficient compression/decompression algorithm for sequences with basic
//! nucleotydes.

use prelude::{K1_SIZE, K_SIZE, COMPRESSED_K1_SIZE};

/// Number of characters fitting inside the byte.
pub const CHARS_PER_BYTE: usize = 4;

/// Compress kmer representation. Output is roughly 2 times smaller,
/// because it needs to store two compressed nodes independently.
///
/// Symbols are put into chunks of 4 and represented as single byte.
#[inline]
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

/// Compress k-mer and return both its compressed representation and compressed
/// reverse complement representation. Return values are
/// (`compressed_repr`, `compressed_reverse_complement`).
#[inline]
pub fn compress_kmer_with_rev_compl(kmer: &[u8]) -> (Vec<u8>, Vec<u8>) {
    assert!(kmer.len() > 2);
    let compressed_size = 2 * ((kmer.len() as f64 - 1.0) / 4.0).ceil() as usize;
    let mut compressed = Vec::with_capacity(compressed_size);
    let mut reverse = Vec::with_capacity(compressed_size);
    let start_node = &kmer[..kmer.len() - 1];
    let end_node = &kmer[1..];
    compress_node(start_node, &mut compressed);
    compress_node(end_node, &mut compressed);
    let remainder = unsafe { K1_SIZE } % CHARS_PER_BYTE;
    reverse.extend(reverse_compressed_node(&compressed[unsafe{COMPRESSED_K1_SIZE}..], remainder));
    reverse.extend(reverse_compressed_node(&compressed[..unsafe{COMPRESSED_K1_SIZE}], remainder));
    (compressed, reverse)
}

/// Compress node representation. Output is roughly 4 times smaller,
/// uses 2 bits per symbol.
///
/// Supports only A, C, T, G as it's alphabet.
#[inline]
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
#[inline]
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
#[inline]
pub fn decompress_kmer(kmer: &[u8]) -> Vec<u8> {
    let mut output: Vec<u8> = Vec::with_capacity(unsafe { K_SIZE });
    let slice_ = &kmer[..unsafe { COMPRESSED_K1_SIZE }];
    let dec = decompress_node(slice_);
    output.extend_from_slice(&dec);
    output.push(get_last_char_from_node(&kmer[unsafe { COMPRESSED_K1_SIZE }..]));
    output
}

#[inline]
fn get_last_char_from_node(node: &[u8]) -> u8 {
    let padding = unsafe { K1_SIZE } % CHARS_PER_BYTE;
    let last_byte: u8 = node[node.len() - 1];
    let padding = (CHARS_PER_BYTE - padding) % CHARS_PER_BYTE;
    decompress_char(last_byte, padding) as u8
}

/// Change a single character in the chunk at the given offset.
#[inline]
pub fn change_char_in_chunk(mut chunk: u8, offset: usize, to: u8) -> u8 {
    let mask = 0b11111100 << (2 * offset);
    let compressed_char = encode_fasta_symbol(to, 0u8) << (2 * offset);
    chunk &= mask;
    chunk |= compressed_char;
    chunk
}

/// Reverse compressed node. Remainder size is the number of bits in the last
/// block.
#[inline]
pub fn reverse_compressed_node(compr: &[u8], remainder_size: usize) -> Vec<u8> {
    let padding = ((CHARS_PER_BYTE - remainder_size) % CHARS_PER_BYTE) * 2;
    let mut reverse = Vec::from(compr);
    let last_byte = reverse.len() - 1;
    shift_right_bit_array(&mut reverse, padding);
    // reverse all bytes in the node
    reverse.reverse();
    // reverse symbols in each byte
    for x in &mut reverse {
        *x = (*x & 0xF0) >> 4 | (*x & 0x0F) << 4;
        *x = (*x & 0xCC) >> 2 | (*x & 0x33) << 2;
        // make it complementary
        *x = !*x;
    }
    // force zero padding
    reverse[last_byte] &= !(((1u8 << padding) - 1) as u8);
    reverse
}

/// Extend edge with the given data. Edge should be compressed, whereas data
/// should be uncompressed.
#[inline]
pub fn extend_edge(edge: &[u8], with: &[u8]) -> Vec<u8> {
    let padding = edge[0];
    let mut vec: Vec<u8> = Vec::new();
    vec.extend_from_slice(edge);
    let mut new_remainder = Vec::new();
    if padding != 0 {
        let padding = (CHARS_PER_BYTE - edge[0] as usize) % CHARS_PER_BYTE;
        new_remainder.extend_from_slice(&decode_compressed_chunk(unwrap!(vec.pop())));
        new_remainder.truncate(padding);
    }
    new_remainder.extend_from_slice(with);
    let compressed = compress_edge(&new_remainder);
    vec[0] = compressed[0];
    vec.extend_from_slice(&compressed[1..]);
    vec
}

/// Change last character in the compressed edge representation.
#[inline]
pub fn change_last_char_in_edge(edge: &[u8], to: u8) -> Vec<u8> {
    let mut output = edge.to_vec();
    let padding = output[0] as usize;
    // mask for zeroing out the last char
    let len = output.len() - 1;
    let last_byte = change_char_in_chunk(output[len], padding, to);
    output[len] = last_byte;
    output
}

/// Append a single character to the compressed edge.
#[inline]
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
#[inline]
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
#[inline]
pub fn compress_edge(edge: &[u8]) -> Vec<u8> {
    assert!(edge.len() > 0);
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
#[inline]
pub fn decompress_edge(edge: &[u8]) -> Vec<u8> {
    let padding = edge[0] as usize;
    let mut output: Vec<u8> = Vec::with_capacity(((edge.len() - 1) * CHARS_PER_BYTE) - padding);
    let slice_ = &edge[1..];
    for chunk in slice_ {
        output.extend_from_slice(&decode_compressed_chunk(*chunk));
    }
    let len = output.len() - padding;
    output.truncate(len);
    output
}

/// Get last character from the compressed edge.
#[inline]
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
///
/// Note that this function accepts only 4 basic nucleotides as the symbol
/// input, namely `A`, `C`, `G`, `T`. Any other symbol will result in
/// **undefined behavior**. Compression will turn the given symbol into two-bit
/// representation, looking as follows:
///
/// * `A -> 00`
/// * `C -> 01`
/// * `G -> 10`
/// * `T -> 11`
///
/// Function will perform a double left shift and append the compressed symbol
/// as two LSBs. It is callees responsibility to ensure that there is enough
/// place for the new symbol in the carrier, otherwise some information will be
/// lost.
///
/// # Example
/// ```
/// use katome::compress::encode_fasta_symbol;
/// let vec = vec![b'A', b'C', b'G', b'T'];
/// let mut result_as_vec = vec![];
/// let mut result_as_block = 0u8;
/// for v in vec {
///     result_as_vec.push(encode_fasta_symbol(v, 0u8));
///     result_as_block = encode_fasta_symbol(v, result_as_block);
/// }
/// assert_eq!(result_as_vec, vec![0, 1, 2, 3]);
/// assert_eq!(result_as_block, 0b00011011);
/// ```
pub fn encode_fasta_symbol(mut symbol: u8, mut carrier: u8) -> u8 {
    // please note that the actual implementation is an optimized version of
    // algorithm, which can be best described by the following naive approach:
    // let x = carrier << 2;
    // match symbol {
    //     b'A' => x,
    //     b'C' => x | 1,
    //     b'G' => x | 2,
    //     b'T' => x | 3,
    //     u => // undefined behavior here
    // }

    // make room for the new symbol
    carrier <<= 2;

    // make 'A' 0
    symbol -= b'A';
    // shift so that second bit is first
    symbol >>= 1;
    // first bit = ~C . D
    // second bit = C + A
    // where:
    // A = 5th bit of the original symbol
    // C = 3rd bit of the original symbol
    // D = 2nd bit of the original symbol
    let c_masked = (symbol & 2) >> 1;
    let a_masked = (symbol & 8) >> 3;
    let d_masked = symbol & 1;
    let first_bit = (c_masked ^ 1) & d_masked;
    let second_bit = c_masked | a_masked;
    carrier | ((second_bit << 1) | first_bit)
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

/// Shift compressed representation to the left. This shift loses information,
/// so make sure that you only shift with proper padding. Can be used to align
/// compressed representation to the left.
#[inline]
pub fn shift_left_bit_array(input: &mut [u8], mut shift_val: usize) {
    shift_val %= 8;
    if shift_val == 0 {
        return;
    }
    let mut old_tmp = 0u8;
    let mut new_tmp;
    let shift_remainder = 8 - shift_val;
    let mask = (((1u8 << shift_val) - 1) << shift_remainder) as u8;
    for byte in input.iter_mut().rev() {
        new_tmp = *byte & mask;
        *byte <<= shift_val;
        *byte |= old_tmp >> shift_remainder;
        old_tmp = new_tmp;
    }
}

/// Shift compressed representation to the right. This shift loses information,
/// so make sure that you only shift with proper padding. This function is
/// usually used to align the underlying representation during calculation of
/// reverse of the compressed k-mer.
#[inline]
pub fn shift_right_bit_array(input: &mut [u8], mut shift_val: usize) {
    shift_val %= 8;
    if shift_val == 0 {
        return;
    }
    let mut old_tmp = 0u8;
    let mut new_tmp;
    let shift_remainder = 8 - shift_val;
    let mask = ((1u8 << shift_val) - 1) as u8;
    for byte in input.iter_mut() {
        new_tmp = *byte & mask;
        *byte >>= shift_val;
        *byte |= old_tmp << shift_remainder;
        old_tmp = new_tmp;
    }
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

    #[test]
    fn extend_edge_test() {
        // pad 1
        let compressed = compress_edge(b"AGGT");
        assert_eq!(vec![0u8, 0b00101011], compressed);
        let added = extend_edge(&compressed, b"GTC");
        assert_eq!(vec![1u8, 0b00101011, 0b10110100], added);

        // pad 2
        let compressed = compress_edge(b"AGGTCGG");
        assert_eq!(vec![1u8, 0b00101011, 0b01101000], compressed);
        let added = extend_edge(&compressed, b"GTC");
        assert_eq!(vec![2u8, 0b00101011, 0b01101010, 0b11010000], added);

        // pad 3
        let compressed = compress_edge(b"AGGTCG");
        assert_eq!(vec![2u8, 0b00101011, 0b01100000], compressed);
        let added = extend_edge(&compressed, b"GTC");
        assert_eq!(vec![3u8, 0b00101011, 0b01101011, 0b01000000], added);

        // pad 0
        let compressed = compress_edge(b"AGGTC");
        assert_eq!(vec![3u8, 0b00101011, 0b01000000], compressed);
        let added = extend_edge(&compressed, b"GTC");
        assert_eq!(vec![0u8, 0b00101011, 0b01101101], added);
    }

    #[test]
    fn shifts_right() {
        let v = vec![0b00101011, 0b01000000];
        let mut v1 = v.clone();
        shift_right_bit_array(&mut v1, 0);
        assert_eq!(v1, v);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 1);
        assert_eq!(v1, vec![0b00010101, 0b10100000]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 2);
        assert_eq!(v1, vec![0b00001010, 0b11010000]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 3);
        assert_eq!(v1, vec![0b00000101, 0b01101000]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 4);
        assert_eq!(v1, vec![0b00000010, 0b10110100]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 5);
        assert_eq!(v1, vec![0b00000001, 0b01011010]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 6);
        assert_eq!(v1, vec![0b00000000, 0b10101101]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 7);
        assert_eq!(v1, vec![0b00000000, 0b01010110]);
        v1 = v.clone();
        shift_right_bit_array(&mut v1, 8);
        assert_eq!(v1, v);
    }

    #[test]
    fn shifts_left() {
        let v = vec![0b00101011, 0b01000001];
        let mut v1 = v.clone();
        shift_left_bit_array(&mut v1, 0);
        assert_eq!(v1, v);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 1);
        assert_eq!(v1, vec![0b01010110, 0b10000010]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 2);
        assert_eq!(v1, vec![0b10101101, 0b00000100]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 3);
        assert_eq!(v1, vec![0b01011010, 0b00001000]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 4);
        assert_eq!(v1, vec![0b10110100, 0b00010000]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 5);
        assert_eq!(v1, vec![0b01101000, 0b00100000]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 6);
        assert_eq!(v1, vec![0b11010000, 0b01000000]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 7);
        assert_eq!(v1, vec![0b10100000, 0b10000000]);
        v1 = v.clone();
        shift_left_bit_array(&mut v1, 8);
        assert_eq!(v1, v);
    }

    #[test]
    fn reverse_complement_2_bytes() {
        // pad 0
        // in:   AGGT
        // out:  ACCT
        let v1 = vec![0b00101011];
        assert_eq!(reverse_compressed_node(&v1, 0), vec![0b00010111]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v1, 0), 0), v1);

        // pad 1
        // in: AGGT GTC
        // out: GACA CCT
        let v2 = vec![0b00101011, 0b10110100];
        assert_eq!(reverse_compressed_node(&v2, 3), vec![0b10000100, 0b01011100]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v2, 3), 3), v2);

        // pad 2
        // in: AGGT GT
        // out: ACAC CT
        let v3 = vec![0b00101011, 0b10110000];
        assert_eq!(reverse_compressed_node(&v3, 2), vec![0b00010001, 0b01110000]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v3, 2), 2), v3);

        // pad 3
        // in: AGGT G
        // out: CACC T
        let v4 = vec![0b00101011, 0b10000000];
        assert_eq!(reverse_compressed_node(&v4, 1), vec![0b01000101, 0b11000000]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v4, 1), 1), v4);
    }

    #[test]
    fn reverse_complement_3_bytes() {
        // pad 0
        // in:   GTCA AATT CCCG
        // out:  CGGG AATT TGAC
        let v1 = vec![0b10110100, 0b00001111, 0b01010110];
        assert_eq!(reverse_compressed_node(&v1, 0), vec![0b01101010, 0b00001111, 0b11100001]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v1, 0), 0), v1);

        // pad 1
        // in:   GTCA AATT CCC
        // out:  GGGA ATTT GAC
        let v2 = vec![0b10110100, 0b00001111, 0b01010100];
        assert_eq!(reverse_compressed_node(&v2, 3), vec![0b10101000, 0b00111111, 0b10000100]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v2, 3), 3), v2);

        // pad 2
        // in:   GTCA AATT CC
        // out:  GGAA TTTG AC
        let v3 = vec![0b10110100, 0b00001111, 0b01010000];
        assert_eq!(reverse_compressed_node(&v3, 2), vec![0b10100000, 0b11111110, 0b00010000]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v3, 2), 2), v3);

        // pad 3
        // in:   GTCA AATT C
        // out:  GAAT TTGA C
        let v4 = vec![0b10110100, 0b00001111, 0b01000000];
        assert_eq!(reverse_compressed_node(&v4, 1), vec![0b10000011, 0b11111000, 0b01000000]);
        assert_eq!(reverse_compressed_node(&reverse_compressed_node(&v4, 1), 1), v4);
    }
}
