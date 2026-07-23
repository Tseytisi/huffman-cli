use crate::tree::HuffmanTree;

pub fn decode_data(
    input_data: &Vec<u8>,
    tree: &HuffmanTree<u64>,
    bits: u32,
    data_offset: u32,
    tree_depth: u32,
) -> Result<Vec<u8>, String> {
    if bits == 0 {
        return Err(String::from("Illegal chunk size"));
    } else if tree_depth == 0 {
        return Err(String::from("Illegal tree depth"));
    } else if data_offset >= input_data.len() as u32 {
        return Err(String::from("Data offset is past end of data"));
    }

    let mut output = Vec::new();

    let mut input_buffer = 0u64;
    let mut input_buffer_size = 0;
    let mut output_buffer = 0u64;
    let mut output_buffer_size = 0;
    for byte in &input_data[(data_offset as usize)..] {
        // Load the next byte into the buffer
        input_buffer <<= 8;
        input_buffer |= *byte as u64;
        input_buffer_size += 8;

        // We start looking as soon as we have enough for the longest path in the tree, so we always
        // get something back as long as the tree is a valid huffman tree
        while input_buffer_size >= tree_depth {
            // Get first value associated with the buffer's contents
            if let Some((value, path_remainder)) = tree.get_value_along_path(input_buffer, input_buffer_size) {
                let path_length = input_buffer_size - path_remainder;
                // If this new value does not fit in our output buffer anymore, we don't store it
                if output_buffer_size + bits > 64 {
                    break;
                }
                input_buffer_size -= path_length;
                output_buffer = output_buffer << bits;
                output_buffer_size += bits;
                output_buffer = output_buffer | *value;
            } else {
                return Err(format!("No returned value for buffer 0x{:x} with length {}",
                                   (input_buffer & !(u64::MAX << input_buffer_size)), input_buffer_size));
            }
        }

        while output_buffer_size >= 8 {
            output_buffer_size -= 8;
            let next_value = ((output_buffer >> output_buffer_size) & 0xFF) as u8;
            output_buffer = output_buffer & !(u64::MAX << output_buffer_size);
            output.push(next_value);
        }
    }

    // After the input bytes run out, we still need to clear out the buffer
    while input_buffer_size > 0 {
        if let Some((value, path_remainder)) = tree.get_value_along_path(input_buffer, input_buffer_size) {
            let path_length = input_buffer_size - path_remainder;
            input_buffer_size -= path_length;
            output_buffer <<= bits;
            output_buffer_size += bits;
            output_buffer |= value;
        } else {
            break;
        }
    }

    while output_buffer_size >= 8 {
        output_buffer_size -= 8;
        let next_value = ((output_buffer >> output_buffer_size) & 0xFF) as u8;
        output_buffer = output_buffer & !(u64::MAX << output_buffer_size);
        output.push(next_value);
    }

    Ok(output)
}