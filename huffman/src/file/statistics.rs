use crate::file::{PlainFile, HuffmanFile};
use crate::FILE_HEADER;

/// File statistics after encoding/decoding. Contains things like file sizes, tree depth, compression
/// ratio, etc.
#[derive(Debug)]
pub struct Statistics {
    /// Size in bytes of the encoded file, including encoded data, serialised tree, and file header.
    pub encoded_file_size: u64,

    /// Size in bytes of the encoded data. This does not include the tree.
    pub encoded_data_size: u64,

    /// Number of bits after the start of the encoded file until the encoded data section starts,
    /// assuming that the tree is stored within the file.
    /// The data before this offset is the header and the tree.
    pub encoded_data_offset: u32,

    /// Size in bytes of the serialised tree. This is the size of the tree when it is stored inside
    /// the encoded file.
    pub serialised_tree_size: u32,

    /// Size in bytes of the original or decoded data.
    pub decoded_file_size: u64,

    /// Depth of the encoding tree. This is the same as the longest encoded value in bits.
    pub encoding_tree_depth: u32,

    /// Number of values present in the tree. This is equal to or less than `2 ^ chunk_size`.
    pub unique_values: u32,

    /// Ratio of values in the tree to possible values with the chunk size. `1` means every possible
    /// value appears in the tree. `0.50` means 50%, etc.
    pub covered_values_ratio: f32,

    /// Size in bits of the data chunks that are taken to encode the file.
    pub chunk_size: u32,

    /// Number of chunks in the decoded file. If `chunk_size` is `8`, this is the same as `decoded_file_size`.
    pub chunk_count: u64,

    /// Compression ratio of the entire file. A value above 1 indicates that the encoded file is
    /// smaller than the original.
    pub file_compression_ratio: f32,

    /// Compression ratio of the data. This is the actual compression achieved by the algorithm, but
    /// since the data is unusable without the tree, this value does not give a good indication of
    /// how much space is saved.
    pub data_compression_ratio: f32,
}

impl PlainFile {
    /// Generates [Statistics] about the file if it has already been encoded.
    ///
    /// Returns `None` if the file has not yet been successfully encoded, as encoding data is required
    /// to generate these statistics.
    pub fn generate_statistics(&self) -> Option<Statistics> {
        if self.tree.is_empty() || self.output_data.is_empty() {
            return None;
        }

        // Serialising the tree is a cheaper operation than running 'estimate'...
        let serialised_tree_size = if let Ok(ser_tree) = self.tree.serialise(self.bits) {
            ser_tree.len() as u32
        } else {
            return None;
        };
        let decoded_file_size = self.input_data.len() as u64;
        let encoded_data_size = self.output_data.len() as u64;
        let encoded_file_size = encoded_data_size + serialised_tree_size as u64 + FILE_HEADER.len() as u64;
        let encoded_data_offset = (encoded_file_size - encoded_data_size) as u32;
        let encoding_tree_depth = self.tree.calculate_depth();
        let unique_values = self.tree.get_values().len() as u32;
        let covered_values_ratio = unique_values as f32 / 2f32.powi(self.bits as i32);
        let chunk_count = (decoded_file_size * 8) / (self.bits as u64);
        let file_compression_ratio = decoded_file_size as f32 / encoded_file_size as f32;
        let data_compression_ratio = decoded_file_size as f32 / encoded_data_size as f32;

        Some(Statistics {
            encoded_file_size,
            encoded_data_size,
            encoded_data_offset,
            serialised_tree_size,
            decoded_file_size,
            encoding_tree_depth,
            unique_values,
            covered_values_ratio,
            chunk_size: self.bits,
            chunk_count,
            file_compression_ratio,
            data_compression_ratio,
        })
    }
}

impl HuffmanFile {
    /// Generates [Statistics] about the file if it has already been decoded.
    ///
    /// Returns `None` if the file has not yet been successfully decoded, as decoding data is required
    /// to generate these statistics.
    pub fn generate_statistics(&self) -> Option<Statistics> {
        if self.tree.is_empty() || self.output_data.is_empty() {
            return None
        }

        let decoded_file_size = self.output_data.len() as u64;
        let encoded_data_offset = self.data_start_offset;
        let encoded_data_size = self.input_data.len() as u64 - encoded_data_offset as u64;
        let serialised_tree_size = if self.data_start_offset < FILE_HEADER.len() as u32 + 1 {
            if let Ok(tree) = self.tree.serialise(self.bits) {
                tree.len() as u32
            } else {
                0
            }
        } else {
            encoded_data_offset - (FILE_HEADER.len() as u32)
        };
        let encoded_file_size = encoded_data_size + serialised_tree_size as u64 + FILE_HEADER.len() as u64;
        let encoding_tree_depth = self.tree.calculate_depth();
        let unique_values = self.tree.get_values().len() as u32;
        let covered_values_ratio = unique_values as f32 / 2f32.powi(self.bits as i32);
        let chunk_count = (decoded_file_size * 8) / (self.bits as u64);
        let file_compression_ratio = decoded_file_size as f32 / encoded_file_size as f32;
        let data_compression_ratio = decoded_file_size as f32 / encoded_data_size as f32;

        Some(Statistics {
            encoded_file_size,
            encoded_data_size,
            encoded_data_offset,
            serialised_tree_size,
            decoded_file_size,
            encoding_tree_depth,
            unique_values,
            covered_values_ratio,
            chunk_size: self.bits,
            chunk_count,
            file_compression_ratio,
            data_compression_ratio,
        })
    }
}