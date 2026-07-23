//! # Huffman
//! Huffman is a library that implements the Huffman compression algorithm in Rust. It supports
//! encoding and decoding of the data, can export the encoded data both with and without encoding
//! tree, and allows you to estimate the encoded file size before encoding.
//!
//! I wrote this library in combination with its command-line wrapper. Though the library is intended
//! for use without the wrapper. The wrapper can be found [on Github](https://github.com/Tseytisi/huffman-cli).
//!
//! ## Modules
//! This library consists of two main modules.
//!
//! ### Tree
//! The [tree] module

pub mod tree;
pub mod file;
mod decode;
mod encode;
mod calculate;

/// File header for exported file, can be any length
const FILE_HEADER: [u8; 4] = [0x48, 0x55, 0x46, 0x31];

#[cfg(test)]
mod tests {
    static LOREM_IPSUM: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. \
Maecenas mattis est sed felis interdum dapibus. Phasellus faucibus, sapien sed bibendum lobortis, \
lacus tortor elementum ligula, non dignissim nulla nulla sed risus. Sed egestas viverra auctor. \
Quisque molestie pulvinar augue a molestie. Integer at nulla eget nisl aliquam ultricies. Donec \
ornare elit non neque tempor, id auctor ipsum aliquet. Praesent cursus, ante non iaculis lobortis, \
magna lorem aliquam felis, non volutpat nulla augue id leo. Phasellus nec dapibus tortor.

Proin eros nunc, laoreet sed accumsan quis, dictum ut nisi. Nullam vitae nisl vehicula, dictum \
tortor ac, consectetur nunc. Sed nisi sem, malesuada vehicula interdum eget, pretium quis velit. \
Vestibulum eget tincidunt metus. Proin placerat diam a lectus commodo, sit amet rhoncus tellus \
interdum. Fusce sollicitudin sed ex id viverra. Duis a mauris eleifend leo lacinia bibendum. \
Sed ex orci, pulvinar eu aliquam vel, suscipit non mauris. Pellentesque sed laoreet augue, sit amet \
facilisis ligula. Vestibulum id aliquam neque. Nulla placerat lorem turpis, sed tincidunt ligula \
mattis in. Vestibulum viverra porta libero, id vestibulum lorem consequat non. Maecenas fringilla \
sollicitudin tellus, eu commodo velit rutrum sed. Integer ut risus orci. Suspendisse malesuada, \
odio a commodo consectetur, lectus ante tempor ex, sed bibendum erat erat vel risus. \
Vestibulum lacinia vitae justo non viverra.

Etiam congue urna urna, vitae dictum lectus faucibus ut. Sed lacinia augue ut erat porta, sit amet \
facilisis felis laoreet. Duis nibh dui, venenatis eu ipsum vel, sodales volutpat nisi. Nunc luctus \
ultricies nunc, at feugiat ante. Donec semper lacus et mauris pellentesque, in interdum felis \
maximus. Etiam viverra diam et enim dictum, quis vehicula leo pulvinar. Nullam et velit eu elit \
pulvinar scelerisque. Aliquam euismod mauris ac elit mollis laoreet. Nulla tincidunt leo maximus \
odio faucibus porttitor. Pellentesque habitant morbi tristique senectus et netus et malesuada fames \
ac turpis egestas. Praesent eget leo tempor, fringilla urna eu, mattis sapien. Morbi commodo magna \
in dui eleifend, et fermentum urna pharetra. Nulla facilisi. Curabitur efficitur odio sit amet \
magna aliquet mattis.xx";

    use crate::file::{HuffmanFile, PlainFile};

    #[test]
    fn encode_decode_default_chunk_size() {
        let data = String::from(LOREM_IPSUM).into_bytes();
        let data_orig = data.clone();

        let mut plain_file = PlainFile::new(data);
        assert!(plain_file.build_tree().is_ok(), "Failed to build tree");
        assert!(plain_file.encode().is_ok(), "Failed to encode");
        let boxed_output = plain_file.get_file_data();
        assert!(boxed_output.is_some(), "File data is none");
        let encoded_file = boxed_output.unwrap();

        let maybe_file = HuffmanFile::new(encoded_file);
        assert!(maybe_file.is_ok(), "HuffmanFile could not be created");
        let mut huffman_file = maybe_file.unwrap();

        let original_encoding_map = plain_file.get_tree().unwrap().generate_encoding_map();
        let imported_encoding_map = huffman_file.get_tree().unwrap().generate_encoding_map();
        assert_eq!(&original_encoding_map, &imported_encoding_map, "Encoding maps were unequal");

        assert!(huffman_file.decode().is_ok(), "Failed to decode");
        assert_eq!(&data_orig, huffman_file.get_output().unwrap(), "Original data and decoded data differ");
    }

    #[test]
    fn encode_decode_odd_chunk_size() {
        let data = String::from(LOREM_IPSUM).into_bytes();
        let data_orig = data.clone();

        let mut plain_file = PlainFile::new(data);
        plain_file.set_chunk_size(5);
        assert!(plain_file.has_valid_chunk_size(), "Chunk size invalid for this data");
        assert!(plain_file.build_tree().is_ok(), "Failed to build tree");
        assert!(plain_file.encode().is_ok(), "Failed to encode");
        let boxed_output = plain_file.get_file_data();
        assert!(boxed_output.is_some(), "File data is none");
        let encoded_file = boxed_output.unwrap();

        let maybe_file = HuffmanFile::new(encoded_file);
        assert!(maybe_file.is_ok(), "HuffmanFile could not be created");
        let mut huffman_file = maybe_file.unwrap();
        assert_eq!(5, huffman_file.get_chunk_size(), "Reported chunk size is not correct");

        let original_encoding_map = plain_file.get_tree().unwrap().generate_encoding_map();
        let imported_encoding_map = huffman_file.get_tree().unwrap().generate_encoding_map();
        assert_eq!(&original_encoding_map, &imported_encoding_map, "Encoding maps were unequal");

        assert!(huffman_file.decode().is_ok(), "Failed to decode");
        assert_eq!(&data_orig, huffman_file.get_output().unwrap(), "Original data and decoded data differ");
    }

    #[test]
    fn encode_decode_separate() {
        let data = String::from(LOREM_IPSUM).into_bytes();
        let data_orig = data.clone();

        let mut plain_file = PlainFile::new(data);
        assert!(plain_file.build_tree().is_ok(), "Failed to build tree");
        assert!(plain_file.encode().is_ok(), "Failed to encode");
        let boxed_output = plain_file.get_output();
        let boxed_tree = plain_file.get_tree();
        assert!(boxed_output.is_some(), "File data is none");
        assert!(boxed_tree.is_some(), "File tree is none");
        let encoded_file = boxed_output.unwrap();
        let tree = boxed_tree.unwrap();

        let mut huffman_file = HuffmanFile::from_separate_parts(
            encoded_file.clone(), tree.clone(), plain_file.get_chunk_size());

        let original_encoding_map = plain_file.get_tree().unwrap().generate_encoding_map();
        let imported_encoding_map = huffman_file.get_tree().unwrap().generate_encoding_map();
        assert_eq!(&original_encoding_map, &imported_encoding_map, "Encoding maps were unequal");

        assert!(huffman_file.decode().is_ok(), "Failed to decode");
        assert_eq!(&data_orig, huffman_file.get_output().unwrap(), "Original data and decoded data differ");
    }
}
