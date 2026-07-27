use std::collections::HashMap;
use std::path::PathBuf;
use crate::tree::HuffmanTree;
use crate::{calculate, encode};
use crate::encode::encode_data;
use crate::decode::decode_data;

pub mod predict;
pub mod statistics;

/// Container that stores (file) data that has already been encoded by this software. After creating
/// this struct, [decode](HuffmanFile::decode) must be called before reading the decoded data through
/// [get_output](HuffmanFile::get_output).
pub struct HuffmanFile {
    tree: HuffmanTree<u64>,
    data_start_offset: u32,
    input_data: Vec<u8>,
    output_data: Vec<u8>,
    bits: u32,
    tree_depth: u32,
}

/// Container that stores (file) data that needs to be encoded. After creating this struct, functions
/// [build_tree](PlainFile::build_tree) and [encode](PlainFile::encode) must be called before the
/// encoded data can be read through [get_output](PlainFile::get_output) or
/// [get_file_data](PlainFile::get_file_data),
/// or written to disk through [export_file](PlainFile::export_file).
pub struct PlainFile {
    tree: HuffmanTree<u64>,
    input_data: Vec<u8>,
    output_data: Vec<u8>,
    bits: u32,
}

impl HuffmanFile {
    /// Create a new [HuffmanFile] from file data. This file is expected to have the [correct
    /// file header](crate::FILE_HEADER) and have both the tree and data stored in it.
    ///
    /// If you wish to create a file from separate parts, use [from_separate_parts](HuffmanFile::from_separate_parts).
    pub fn new(data: Vec<u8>) -> Result<Self, String> {
        if !data.starts_with(&crate::FILE_HEADER) {
            return Err(String::from("Invalid file header"));
        }

        return HuffmanFile::new_ignore_header(data, crate::FILE_HEADER.len());
    }

    /// Create a new [HuffmanFile] from a file on disk. This file is expected to have the [correct
    /// file header](crate::FILE_HEADER) and have both the tree and data stored in it.
    ///
    /// This function simply reads the file and then calls [new](HuffmanFile::new)
    pub fn from_filepath(filepath: &str) -> Result<Self, String> {
        if let Ok(data) = std::fs::read(filepath) {
            HuffmanFile::new(data)
        } else {
            Err(String::from("Could not open file"))
        }
    }

    /// Create a new [HuffmanFile] from file data. This file is expected to have both the tree and
    /// data stored in it. This function will not check for a file header. You can give an offset
    /// in bytes, where the first `offset` number of bytes will be skipped when parsing.
    ///
    /// If you wish to create a file from separate parts, use [from_separate_parts](HuffmanFile::from_separate_parts).
    pub fn new_ignore_header(data: Vec<u8>, offset: usize) -> Result<Self, String> {
        // Get the tree from the file
        let maytree = HuffmanTree::deserialise(&data, offset);
        let tree: HuffmanTree<u64>;
        let data_offset: u32;
        match maytree {
            Ok(t) => {
                tree = t.0;
                data_offset = t.1;
            },
            Err(e) => return Err(format!("Error while deserialising tree: {}", e)),
        }

        let tree_depth: u32;
        let bits: u32;
        if let Some(b) = data.get(offset + 2) {
            bits = *b as u32;
        } else {
            return Err(String::from("Could not determine data chunk size"));
        }
        if let Some(t) = data.get(offset) {
            tree_depth = *t as u32;
        } else {
            return Err(String::from("Could not determine tree depth"));
        }

        Ok(HuffmanFile {
            input_data: data,
            tree,
            output_data: Vec::new(),
            bits,
            data_start_offset: data_offset,
            tree_depth,
        })
    }

    /// Create a new [HuffmanFile] from separate objects.
    /// Warning: this function does not verify that the tree is actually valid or usable for the input data,
    /// or that the chunk size is correct.
    pub fn from_separate_parts(encoded_data: Vec<u8>, tree: HuffmanTree<u64>, chunk_size: u32) -> Self {
        let tree_depth = (&tree).calculate_depth();
        HuffmanFile {
            tree,
            data_start_offset: 0,
            input_data: encoded_data,
            output_data: Vec::new(),
            bits: chunk_size,
            tree_depth,
        }
    }

    /// Create a new [HuffmanFile] from the encoded data and the serialised tree data. This method
    /// is very similar to [from_separate_parts](HuffmanFile::from_separate_parts), but might be
    /// more convenient.
    pub fn from_raw_data(encoded_data: Vec<u8>, tree_data: Vec<u8>) -> Result<Self, String> {
        return match HuffmanTree::deserialise(&tree_data, 0) {
            Ok(tree) => {
                Ok(HuffmanFile {
                    tree_depth: tree.0.calculate_depth(),
                    tree: tree.0,
                    data_start_offset: 0,
                    input_data: encoded_data,
                    output_data: Vec::new(),
                    bits: tree.1,
                })
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    /// Decode the data contained in the `input` with the provided tree. If the tree is the correct
    /// tree for the given data, and the file contents are intact, this method should always succeed.
    /// Returns an empty [Ok] upon success, after which the output is available through, among others,
    /// [get_output](HuffmanFile::get_output). If this method fails, it returns [Err] containing a
    /// string describing the error that occurred.
    pub fn decode(&mut self) -> Result<(), String> {
        match decode_data(&self.input_data, &self.tree, self.bits, self.data_start_offset, self.tree_depth) {
            Ok(data) => {
                self.output_data = data;
                Ok(())
            },
            Err(e) => {
                Err(e)
            },
        }
    }

    /// Get a reference to the [HuffmanTree] contained within this file
    pub fn get_tree(&self) -> Option<&HuffmanTree<u64>> {
        if self.tree.is_empty() {
            None
        } else {
            Some(&self.tree)
        }
    }

    /// Get a reference to the encoded data in this file
    pub fn get_input(&self) -> &Vec<u8> {
        &self.input_data
    }

    /// Get the chunk size of the encoded data
    pub fn get_chunk_size(&self) -> u32 {
        self.bits
    }

    /// Get the data offset of the encoded data. In regular files, the tree is stored before the data
    /// so this points to the byte of the input data that marks the start of the encoded data.
    pub fn get_data_offset(&self) -> u32 {
        self.data_start_offset
    }

    /// Get a reference to the decoded output data of this file, if the function
    /// [decode](HuffmanFile::decode) succeeded.
    /// If not, or it has not been called yet, this function returns [None].
    pub fn get_output(&self) -> Option<&Vec<u8>> {
        if self.output_data.is_empty() {
            None
        } else {
            Some(&self.output_data)
        }
    }
}

impl PlainFile {
    /// Create a new [PlainFile] by providing the raw un-encoded data in bytes. This sets the chunk
    /// size to `8` bits, and does not build the tree or encode the data.
    pub fn new(data: Vec<u8>) -> Self {
        PlainFile {
            tree: HuffmanTree::new(),
            input_data: data,
            output_data: Vec::new(),
            bits: 8,
        }
    }

    /// Create a new [PlainFile] by providing a filepath to the file that needs to be encoded. The
    /// file will be read and its contents stored inside this [PlainFile]. If the file is not readable,
    /// this function returns an error.
    pub fn from_filepath(path: &str) -> Result<Self, String> {
        if let Ok(file) = std::fs::read(path) {
            Ok(PlainFile {
                tree: HuffmanTree::new(),
                input_data: file,
                output_data: Vec::new(),
                bits: 8,
            })
        } else {
            Err(format!("Error reading file at '{}'", path))
        }
    }

    /// Set the number of bits that should be taken for each 'chunk' of the file. If this value is
    /// different from the old value, this will reset the currently stored encoding tree and output
    /// data.
    pub fn set_chunk_size(&mut self, bits: u32) {
        if self.bits != bits {
            self.bits = bits;
            self.output_data = Vec::new();
            self.tree = HuffmanTree::new();
        }
    }

    /// Check whether the set chunk size is valid to encode this file with. More precisely, checks
    /// whether the chunk size evenly divides the number of bits in the input data. If this function
    /// returns `false`, calls to [encode](PlainFile::encode) will fail.
    pub fn has_valid_chunk_size(&self) -> bool {
        return (self.input_data.len() as u32 * 8) % self.bits == 0
    }

    /// Build the tree for the data inside. Returns an empty [Ok] if this action succeeds, and an [Err] if
    /// it didn't. This function can only fail if the number of bits in the input data is not evenly
    /// divisible by the set value for `bits`. You can check this beforehand by calling
    /// [has_valid_chunk_size](PlainFile::has_valid_chunk_size).
    pub fn build_tree(&mut self) -> Result<(), String> {
        match encode::build_tree(&self.input_data, self.bits) {
            Ok(tree) => {
                self.tree = tree;
                Ok(())
            },
            Err(e) => Err(e)
        }
    }

    /// Encode the input data with the calculated encoding tree. Upon success, this method returns
    /// an empty [Ok]. Returns [Err] if the tree has not been built successfully before this function is
    /// invoked.
    pub fn encode(&mut self) -> Result<(), String> {
        if self.tree.is_empty() {
            return Err(String::from("The encoding tree must be built before this method is invoked"));
        }

        match encode_data(&self.input_data, self.bits, &self.tree) {
            Ok(data) => {
                self.output_data = data;
                Ok(())
            },
            Err(e)  => {
                Err(e)
            }
        }
    }

    /// Return a reference to the tree contained within this [PlainFile]. If no tree has been built,
    /// this method returns [None]
    pub fn get_tree(&self) -> Option<&HuffmanTree<u64>> {
        if !self.tree.is_empty() {
            Some(&self.tree)
        } else {
            None
        }
    }

    /// Create a copy of the tree contained within this file and return it. If no tree has been built,
    /// this method returns [None]. This function is a lot more expensive than [get_tree](PlainFile::get_tree).
    pub fn copy_tree(&self) -> Option<HuffmanTree<u64>> {
        if self.tree.is_empty() {
            None
        } else {
            Some(self.tree.clone())
        }
    }

    /// Return a reference to the data contained within this [PlainFile]. This data is immutable after
    /// initialisation.
    pub fn get_input(&self) -> &Vec<u8> {
        &self.input_data
    }

    /// Return a reference to the encoded data contained within this [PlainFile]. If the input data
    /// has not yet been encoded, this method returns [None].
    pub fn get_output(&self) -> Option<&Vec<u8>> {
        if self.output_data.is_empty() {
            None
        } else {
            Some(&self.output_data)
        }
    }

    /// (Re-)calculate the occurrences of each chunk in the input data.
    /// This is done automatically during the encoding process, and does
    /// not need to be done manually unless this information is required
    /// after the encoding process.
    ///
    /// Returns a map, mapping a value to the number of times it occurs in the input.
    pub fn calculate_occurrences(&self) -> Result<HashMap<u64, u32>, String> {
        calculate::count_chunks(&self.input_data, self.bits)
    }

    /// Exports this [PlainFile] to disk. The resulting file will contain the default header, the
    /// encoding tree, and the encoded data, all in the same file.
    ///
    /// Returns an empty [Ok] upon success. Returns an [Err] if the file could not be written, or
    /// no output data is available.
    pub fn export_file(&self, filepath: &PathBuf) -> Result<(), String> {
        return if let Some(data) = self.get_file_data() {
            return match std::fs::write(filepath, data) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("{}", e))
            };
        } else {
            Err(String::from("No data to output - Encode the data before invoking this method"))
        }
    }

    /// Export this [PlainFile] as a vector of bytes. The resulting data will contain the default header,
    /// the encoding tree, and the encoded data.
    ///
    /// Fails if the output data has not been encoded yet,
    pub fn get_file_data(&self) -> Option<Vec<u8>> {
        if self.output_data.is_empty() {
            return None;
        }

        let mut output: Vec<u8> = Vec::new();
        for byte in crate::FILE_HEADER {
            output.push(byte);
        }

        if let Ok(mut ser_tree) = self.tree.serialise(self.bits) {
            output.append(&mut ser_tree);
        }

        output.append(&mut self.output_data.clone());

        Some(output)
    }

    /// Exports the output of this [PlainFile] to disk. Only the data will be written. Without storing
    /// the tree, this data cannot be decoded.
    ///
    /// Returns an empty [Ok] upon success. Returns an [Err] if the file could not be written, or no output
    /// data is available.
    pub fn export_data_only(&self, filepath: &PathBuf) -> Result<(), String> {
        if self.output_data.is_empty() {
            return Err(String::from("No data to output - Encode the data before invoking this method"));
        }

        return match std::fs::write(filepath, &self.output_data) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("{}", e))
        }
    }

    /// Exports a serialised version of the tree to disk. Only the tree will be written, without header.
    ///
    /// Returns an empty [Ok] upon success. Returns an [Err] if the file could not be written, or no tree
    /// has been built yet.
    pub fn export_tree_only(&self, filepath: &PathBuf) -> Result<(), String> {
        if self.tree.is_empty() {
            return Err(String::from("No tree to output - Build the tree before invoking this method"));
        }
        return match self.tree.serialise(self.bits) {
            Ok(ser_tree) => {
                match std::fs::write(filepath, ser_tree) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(format!("{}", e))
                }
            },
            Err(e) => Err(e),
        }
    }

    /// Get the chunk size this [PlainFile] is set to
    pub fn get_chunk_size(&self) -> u32 {
        self.bits
    }
}
