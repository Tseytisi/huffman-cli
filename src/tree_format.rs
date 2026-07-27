pub fn ascii_tree_formatter<const E: bool>(x: &u64) -> String {
    if E {
        // Escape characters for dot-file output
        if *x == '\\' as u64 {
            String::from("\\\\")
        } else if *x == '"' as u64 {
            String::from("\\\"")
        } else {
            byte_to_ascii(x).to_string()
        }
    } else {
        byte_to_ascii(x).to_string()
    }
}

const MISSING_CHARS: [char; 32] = [
    '€', '≈', '‚', 'ƒ', '„', '…', '†', '‡',
    'ˆ', '‰', 'Š', '‹', 'Œ', '≥', 'Ž', '≤',
    '\u{2425}', // "Symbol for delete form two"
    '‘', '’', '“', '”', '•', '–', '—',
    '˜', '™', 'š', '›', 'œ',
    '\u{2426}', // Symbol for substitute form two"
    'ž', 'Ÿ',
];

// Mostly Windows-1252, but with a few extras thrown in,
// and replacing space, nbsp and soft hyphen
pub fn byte_to_ascii(x: &u64) -> char {
    match x {
        // Printable control characters
        0..=31 => char::from_u32(0x2400 + *x as u32).unwrap(),
        32 => '\u{2423}', // Space: open box
        127 => '\u{2421}', // Printable DEL
        0x80..=0x9f => MISSING_CHARS[*x as usize - 0x80],
        0xa0 => '\u{2420}', // NBSP: symbol for space
        0xad => '\u{2422}', // SHY: 'blank symbol' (b with bar)
        // Regular unicode
        0x21..=0x7e | 0xa1..=0xac | 0xae..=0xff =>
            char::from_u32(*x as u32).unwrap(),
        256.. => panic!("Invalid input to byte_to_ascii"),
    }
}

pub fn hex_tree_formatter<const C: usize>(x: &u64) -> String {
    format!("0x{:0C$x}", x)
}

pub fn initialised_hex_tree_formatter(chunk_size: u32) -> fn(&u64) -> String {
    let strlen: u32 = ((chunk_size - 1) / 4) + 1;
    // I am somewhat sorry it's this or using Box<dyn Fn(&u64) -> String> everywhere
    match strlen {
        1 => hex_tree_formatter::<1>,
        2 => hex_tree_formatter::<2>,
        3 => hex_tree_formatter::<3>,
        4 => hex_tree_formatter::<4>,
        5 => hex_tree_formatter::<5>,
        6 => hex_tree_formatter::<6>,
        7 => hex_tree_formatter::<7>,
        8 => hex_tree_formatter::<8>,
        9 => hex_tree_formatter::<9>,
        10 => hex_tree_formatter::<10>,
        11 => hex_tree_formatter::<11>,
        12 => hex_tree_formatter::<12>,
        13 => hex_tree_formatter::<13>,
        14 => hex_tree_formatter::<14>,
        15 => hex_tree_formatter::<15>,
        16 => hex_tree_formatter::<16>,
        _ => panic!("Invalid chunk size")
    }
}

pub fn bin_tree_formatter<const C: usize>(x: &u64) -> String {
    format!("0b{:0C$b}", x)
}

pub fn initialised_bin_tree_formatter(chunk_size: u32) -> fn(&u64) -> String {
    // I am a little more sorry
    match chunk_size {
        1 => bin_tree_formatter::<1>,
        2 => bin_tree_formatter::<2>,
        3 => bin_tree_formatter::<3>,
        4 => bin_tree_formatter::<4>,
        5 => bin_tree_formatter::<5>,
        6 => bin_tree_formatter::<6>,
        7 => bin_tree_formatter::<7>,
        8 => bin_tree_formatter::<8>,
        9 => bin_tree_formatter::<9>,
        10 => bin_tree_formatter::<10>,
        11 => bin_tree_formatter::<11>,
        12 => bin_tree_formatter::<12>,
        13 => bin_tree_formatter::<13>,
        14 => bin_tree_formatter::<14>,
        15 => bin_tree_formatter::<15>,
        16 => bin_tree_formatter::<16>,
        17 => bin_tree_formatter::<17>,
        18 => bin_tree_formatter::<18>,
        19 => bin_tree_formatter::<19>,
        20 => bin_tree_formatter::<20>,
        21 => bin_tree_formatter::<21>,
        22 => bin_tree_formatter::<22>,
        23 => bin_tree_formatter::<23>,
        24 => bin_tree_formatter::<24>,
        25 => bin_tree_formatter::<25>,
        26 => bin_tree_formatter::<26>,
        27 => bin_tree_formatter::<27>,
        28 => bin_tree_formatter::<28>,
        29 => bin_tree_formatter::<29>,
        30 => bin_tree_formatter::<30>,
        31 => bin_tree_formatter::<31>,
        32 => bin_tree_formatter::<32>,
        33 => bin_tree_formatter::<33>,
        34 => bin_tree_formatter::<34>,
        35 => bin_tree_formatter::<35>,
        36 => bin_tree_formatter::<36>,
        37 => bin_tree_formatter::<37>,
        38 => bin_tree_formatter::<38>,
        39 => bin_tree_formatter::<39>,
        40 => bin_tree_formatter::<40>,
        41 => bin_tree_formatter::<41>,
        42 => bin_tree_formatter::<42>,
        43 => bin_tree_formatter::<43>,
        44 => bin_tree_formatter::<44>,
        45 => bin_tree_formatter::<45>,
        46 => bin_tree_formatter::<46>,
        47 => bin_tree_formatter::<47>,
        48 => bin_tree_formatter::<48>,
        49 => bin_tree_formatter::<49>,
        50 => bin_tree_formatter::<50>,
        51 => bin_tree_formatter::<51>,
        52 => bin_tree_formatter::<52>,
        53 => bin_tree_formatter::<53>,
        54 => bin_tree_formatter::<54>,
        55 => bin_tree_formatter::<55>,
        56 => bin_tree_formatter::<56>,
        57 => bin_tree_formatter::<57>,
        58 => bin_tree_formatter::<58>,
        59 => bin_tree_formatter::<59>,
        60 => bin_tree_formatter::<60>,
        61 => bin_tree_formatter::<61>,
        62 => bin_tree_formatter::<62>,
        63 => bin_tree_formatter::<63>,
        64 => bin_tree_formatter::<64>,
        _ => panic!("Invalid chunk size")
    }
}
