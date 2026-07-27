use std::collections::HashMap;
use crate::tree::HuffmanTree;
use crate::calculate::*;

pub fn build_tree(data: &Vec<u8>, bits: u32) -> Result<HuffmanTree<u64>, String> {
    let count_map: HashMap<u64, u32>;
    match count_chunks(data, bits) {
        Ok(map) => count_map = map,
        Err(e) => return Err(e),
    }
    //println!("Count bytes: {:?}", &count_map);
    let levels = calculate_levels(count_map);
    //println!("Levels: {:?}", &levels);
    let path_values = calculate_path_values(&levels);
    //println!("Path values: {:?}", &path_values);
    let tree = populate_tree(&path_values, &levels);
    Ok(tree)
}

pub fn encode_data(data: &Vec<u8>, bits: u32, tree: &HuffmanTree<u64>) -> Result<Vec<u8>, String> {
    let encoding_map = tree.generate_encoding_map();

    let mut encoded_data: Vec<u8> = Vec::new();

    // Buffer between the input data byte vector and the chunks that will be used for encoding
    let mut input_buffer = 0u64;
    let mut input_buffer_length = 0;

    // Buffer between the resulting binary values of varying length that are returned from the encoding
    // map, and the output byte vector
    let mut output_buffer = 0u64;
    let mut output_buffer_length = 0;

    for byte in data {
        // Fill the input buffer with the new byte
        input_buffer = (input_buffer << 8) | (*byte as u64);
        input_buffer_length += 8;

        // As long as there are enough bits in the buffer to encode them, we do so
        while input_buffer_length >= bits {
            input_buffer_length -= bits;
            let next_chunk = input_buffer >> input_buffer_length;
            input_buffer = input_buffer & !(u64::MAX << input_buffer_length);
            if let Some((path, level)) = encoding_map.get(&next_chunk) {
                // Level is the same as the tree depth at this value's node, and thus the path length
                // and binary value length
                // Move over the buffer contents to make room for the new data
                //println!("Adding {:0width$b} to buffer for chunk {:x}", path, next_chunk, width=*level as usize);
                output_buffer = output_buffer << *level;
                output_buffer = output_buffer | (*path as u64);
                output_buffer_length += *level;
            } else {
                return Err(format!("No mapping for chunk 0x{:x}", next_chunk));
            }
        }

        while output_buffer_length >= 8 {
            output_buffer_length -= 8;
            let next_byte = (output_buffer >> (output_buffer_length)) as u8;
            encoded_data.push(next_byte);
            output_buffer = output_buffer & !(u64::MAX << output_buffer_length);
        }
    }

    if output_buffer_length > 0 {
        let next_byte = (output_buffer << (8 - output_buffer_length)) as u8;
        encoded_data.push(next_byte);
    }

    Ok(encoded_data)
}

fn populate_tree(path_values: &HashMap<u64, u32>, levels: &HashMap<u64, u32>) -> HuffmanTree<u64> {
    let mut tree = HuffmanTree::new();
    let highest_level = levels.iter().map(|(_v, l)| *l).max().unwrap_or(0);

    for (value, path_value) in path_values {
        if let Some(level) = levels.get(value) {
            let path = *path_value >> (highest_level - *level);
            tree.set(*value, path, *level);
        } else {
            panic!("No level mapping for value {}", value);
        }
    }

    tree
}
