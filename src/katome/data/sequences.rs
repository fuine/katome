const SIZE: usize = 4;

pub fn sequence_to_u64(s: &str) -> u64 {
    let mut encoded: u64 = 0;
    for c in s.chars() {
        println!("{:#010b}", encoded);
        encoded = encode_char(c, encoded);
    }
    encoded
}

// this function needs size info about the padding in the string
pub fn u64_to_sequence(i: u64) -> String {
    // let mut sequence: String = "".to_string();
    let mut v: Vec<u8> = vec![0; SIZE];
    let mut a = i;
    // let mut chr: char = 'a';
    for n in (0..SIZE).rev() {
        let (chr, x) = decode_char(a);
        a = x;
        // sequence.push(chr);
        v[n] = chr;
    }
    String::from_utf8(v).unwrap()
}

fn encode_char(c: char, i: u64) -> u64 {
    let x: u64 = i << 2; // make room for a new character
    match c {
        'a' => x,
        'b' => x | 1,
        'c' => x | 2,
        'd' => x | 3,
        _ => i + 5,
    }
}

fn decode_char(i: u64) -> (u8, u64) {
    let c: u64 = i & 3; // TODO add binary masks in here
    let x = i >> 2;
    match c {
        0 => (65, x),
        1 => (66, x),
        2 => (67, x),
        3 => (68, x),
        _ => (0, x),
    }
}
