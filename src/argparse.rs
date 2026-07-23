use std::path::PathBuf;
use clap::{Parser, Subcommand};

static ABOUT_ENCODE: &str = "Encode a regular file with the Huffman compression algorithm.

If only the input file path is given, this file will be encoded and the data and tree written to
disk together in the same output file.

If the 'tree_path' argument is also given, only the encoded data will be written to the output file,
and the encoding tree will be written to a separate file specified by 'tree_path'.

If no 'output' is given, the output file will be written to the same folder the input file is in,
with the same filename as the input with the addition of an extra file extension. If 'output' is
specified, this is used instead.

With 'chunk_size' it is possible to specify the number of bits the encoder needs to consider as one
'chunk' of the file. Usually, the larger the chunk size is, the smaller the encoded file, but the
larger the encoding tree becomes. For most files, 8 bits (1 byte) is best. If the chunk size chosen
does not evenly divide the bits in the input, the command will fail. Possible chunk sizes can be viewed
with the 'estimate' command.

You can also enable 'use_best_compression', which runs 'estimate' before encoding and chooses the
chunk size that results in the smallest total size. Be aware that 'estimate' is not a cheap operation,
and can cause the execution time to be longer.";

static ABOUT_DECODE: &str = "Decode a Huffman-encoded file and output its original.

There are two ways to encode a file with this software, and how to decode that file depends on the
manner of encoding. The default is to output both the data and the encoding tree to the same file.
This file, on its own, can then be decoded by simply providing the encoded file.

If during encoding the encoding tree was saved to a separate file, both files must be provided.
Huffman-encoded data without an encoding tree is impossible to decode. To provide both the data and
the tree, simply give the data as the 'input' and the tree as the 'tree'. If any file is given for
the tree, the input will be interpreted as raw encoded data.

This software can only decode files that were encoded by this software. The data portion should be
similar to the way other Huffman encoders encode their data, but in order to decode, an encoding tree
for the specific file must be provided. This software stores the encoding tree in an efficient but
custom way. It cannot import encoding trees from other encoding software.

Furthermore, this software does not always produce the same encoding trees for a given input, though
the resulting file sizes are the same.";

static ABOUT_ESTIMATE: &str = "Estimate the resulting encoded file sizes for all possible chunk sizes.

The 'encode' subcommand allows you to specify the 'chunk size' for a file. This greatly influences
the file size of the resulting encoded file. Bigger chunk sizes usually reduce the size of the
encoded data but greatly increase the size of the encoding tree. For most files,
a chunk size of 8 bits (1 byte) is the most efficient. But for, for example, UTF-16 encoded files,
a chunk size of 16 bits may result in a smaller encoded file.

The estimate command allows you to view all possible chunk sizes for the given file, and see the
resulting sizes of the encoded data part, the encoding tree part, and the total file size for a file
with both the data, tree and header.";

/// Command-line program to encode or decode file with the Huffman compression algorithm.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    /// Only print error messages
    #[arg(short = 'S', long)]
    pub silent: bool,

    /// Print information helpful for debugging
    #[arg(short, long, default_value_t = false)]
    pub debug: bool,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Encode a regular file with the Huffman compression algorithm
    #[command(about, long_about = ABOUT_ENCODE, alias = "compress")]
    Encode(EncodeArgs),
    /// Decode a Huffman-encoded file and output its original
    #[command(about, long_about = ABOUT_DECODE, alias = "uncompress")]
    Decode(DecodeArgs),
    /// Estimate the resulting encoded file sizes for all possible chunk sizes
    #[command(about, long_about = ABOUT_ESTIMATE)]
    Estimate(EstimateArgs),
}

/// Encode a regular file with the Huffman compression algorithm
#[derive(Parser, Debug)]
pub struct EncodeArgs {
    /// Path and filename of the file to encode
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    /// Path and filename of the encoding tree; causes the output to be split into separate files
    #[arg(short, long = "tree", value_name = "FILE")]
    pub tree_path: Option<PathBuf>,

    /// Path and filename of the output file, or output data if 'tree' is also given
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Print file compression statistics such as compression ratio
    #[arg(short, long)]
    pub print_statistics: bool,

    /// Number of bits per data chunk to break the original file up in (max 64)
    #[arg(short, long, default_value_t = 8, value_parser = clap::value_parser!(u32).range(2..=64))]
    pub chunk_size: u32,

    /// Estimates result file sizes for all possible chunk sizes before encoding
    #[arg(short = 'b', long, default_value_t = false)]
    pub use_best_compression: bool,
}

/// Decode a Huffman-encoded file back to its original
#[derive(Parser, Debug)]
pub struct DecodeArgs {
    /// Path and filename of the encoded file
    #[arg(value_name = "FILE")]
    pub input: PathBuf,

    /// Path and filename of the encoding tree if it is separate from the input data. This will
    /// cause the input to be treated as data only
    #[arg(short, long = "tree", value_name = "FILE")]
    pub tree_path: Option<PathBuf>,

    /// Path and filename of the output file
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Print file compression statistics such as compression ratio
    #[arg(short, long)]
    pub print_statistics: bool,
}

/// Estimate the resulting file sizes of all possible chunk sizes
#[derive(Parser, Debug)]
pub struct EstimateArgs {
    /// Path and filename of the file to encode
    #[arg(value_name = "FILE")]
    pub input: PathBuf,
}
