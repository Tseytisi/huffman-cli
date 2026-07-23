//! ## Tree
//! This module contains one struct, [HuffmanTree], and many implementations for it.
//! With regular use of the program, you do not need to interact with the encoding trees yourself,
//! only if you wish to save the contents of the tree in a different than standard way.
//!
//! ### Manually creating a new tree
//! If you know the values you want to place in the tree, and where you want to place them, you
//! can build a tree manually. The [HuffmanTree] works with binary paths, where each `0` is a left
//! branch and each `1` is a right branch. This is explained more in-depth in the [set](HuffmanTree::set)
//! function.
//!
//! To create a new tree that looks like this:
//! ```-
//! Level
//!  0         root
//!         0 /   \ 1
//!          /     \
//!  1      A      /\
//!             0 /  \ 1
//!              /    \
//!  2          B      C
//! ```
//!
//! You run
//! ```
//! use huffman::tree::HuffmanTree;
//!
//! fn main() {
//!     let mut tree = HuffmanTree::new();
//!     tree.set('A', 0, 1); // The path to A is '0', at level 1 (path length is also 1)
//! }
//! ```

mod serialise;
use std::cmp::max;
use std::collections::{HashMap, LinkedList};
use std::fmt::{Display, Debug, LowerHex};
use std::hash::Hash;

/// Restricted binary tree implementation specifically designed to efficiently decode huffman-compressed
/// files. Values are stored at leafs only, no node should hold a value.
///
/// To create a new HuffmanTree, create a new, empty tree first, and
#[derive(Debug, PartialEq)]
pub struct HuffmanTree<T> {
    value: Option<T>,
    left: Option<Box<HuffmanTree<T>>>,
    right: Option<Box<HuffmanTree<T>>>,
}

impl<T> HuffmanTree<T> {
    /// Create a new [HuffmanTree] with no value, and no subtrees.
    pub fn new() -> Self {
        HuffmanTree {
            value: None,
            left: None,
            right: None,
        }
    }

    /// Return [true] if this tree is equivalent to `HuffmanTree::new()`, and thus has no value at
    /// the root node, and no subtrees beyond that. If either of these conditions is not met, [true]
    /// is returned, even if none of the child nodes contain any value
    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.left.is_none() && self.right.is_none()
    }

    /// Sets a value at the node reachable by the binary representation of the given `path` variable,
    /// interpreted as a binary value with `level` number of bits. So `path = 1`, `level = 1`
    /// (binary `1`) will put the given value at the node reachable by going `right` from the root node.
    /// `path = 1` `level = 4` (binary `0001`) will put the value at the node reachable by
    /// going `left -> left -> left -> right` from the root node.
    ///
    /// Returns `true` if the set was a success. Returns `false` if the binary representation of the
    /// path requires more bits than the level. The tree is not altered in that case, and `false` is
    /// returned.
    pub fn set(&mut self, value: T, path: u32, level: u32) -> bool {
        return if level == 0 {
            self.value = Some(value);
            true
        } else if path >= 2u32.pow(level) {
            false
        } else {
            if path >= 2u32.pow(level - 1) {
                // Go right
                if self.right.is_none() {
                    self.right = Some(Box::new(HuffmanTree::new()));
                }
                if let Some(ref mut right) = &mut self.right {
                    (*right).set(value, path - 2u32.pow(level - 1), level - 1)
                } else {
                    false
                }
            } else {
                // Go left
                if self.left.is_none() {
                    self.left = Some(Box::new(HuffmanTree::new()));
                }
                if let Some(ref mut left) = &mut self.left {
                    (*left).set(value, path, level - 1)
                } else {
                    false
                }
            }
        }
    }

    /// Return a list of all values stored in this tree, along with the path to that node, and the
    /// tree depth at that node. The path is stored in binary, where the root node starts at the most
    /// significant value, left is a `0` and right is a `1`. Thus, a value `x` stored at a node
    /// reachable from the top at `left -> right -> left -> left` would be returned as `(x, (4, 4))`
    /// since `4` is `0100`, and the tree depth (and number of bits in binary) is `4`.
    pub fn get_values(&self) -> Vec<(&T, (u32, u32))> {
        return self.get_all_values_with_paths(0, 0);
    }

    fn get_all_values_with_paths(&self, path: u32, level: u32) -> Vec<(&T, (u32, u32))> {
        let mut output = Vec::new();
        if let Some(value) = &self.value {
            output.push((value, (invert_binary(path, level), level)));
        }
        if let Some(left) = &self.left {
            output.append(&mut left.get_all_values_with_paths(path, level + 1));
        }
        if let Some(right) = &self.right {
            output.append(&mut right.get_all_values_with_paths(path + 2u32.pow(level), level + 1));
        }

        return output
    }

    /// Export this tree as dot-file. This is a generic function that requires a formatting function
    /// for type T. If your type T implements [Display] or [Debug], consider using
    /// [export_tree_dot_display] or [export_tree_dot_debug].
    pub fn export_tree_dot<F>(&self, format_func: &F) -> String
        where F: Fn(&T) -> String {

        // Start of dot-file
        let mut output = String::from("graph tree {\n");
        // Formatting
        output.push_str("    node [shape=\"none\"]\n");

        // Root node label
        output.push_str("    root [label=\"");
        if let Some(value) = &self.value {
            output.push_str(&format_func(&value));
        } else {
            output.push_str("root");
        }
        output.push_str("\"]\n");

        // If we have a left subtree
        if let Some(boxed_left) = &self.left {
            output.push_str("    root -- _0 [label=\"0\"]\n");
            output.push_str(&boxed_left.tree_to_dot("_0", format_func));
        }
        // If we have a right subtree
        if let Some(boxed_right) = &self.right {
            output.push_str("    root -- _1 [label=\"1\"]\n");
            output.push_str(&boxed_right.tree_to_dot("_1", format_func));
        }

        // End of dot-file
        output.push_str("}");
        output
    }

    fn tree_to_dot<F>(&self, name: &str, format_func: &F) -> String
        where F: Fn(&T) -> String {
        // Create a line for the top node of this tree
        let mut output = format!("    {} [label=\"", name);
        if let Some(value) = &self.value {
            output.push_str(&format_func(value));
        }
        output.push_str("\"]\n");

        // If we have a left subtree
        if let Some(left) = &self.left {
            let new_name = format!("{}0", name);
            output.push_str(&format!("    {name} -- {new_name} [label=\"0\"]\n"));
            output.push_str(&left.tree_to_dot(&new_name, format_func));
        }
        // If we have a right subtree
        if let Some(right) = &self.right {
            let new_name = format!("{}1", name);
            output.push_str(&format!("    {name} -- {new_name} [label=\"1\"]\n"));
            output.push_str(&right.tree_to_dot(&new_name, format_func));
        }

        output
    }

    /// Calculates the depth of the tree
    pub fn calculate_depth(&self) -> u32 {
        let left_depth: u32;
        let right_depth: u32;
        if let Some(left) = &self.left {
            left_depth = 1 + left.calculate_depth();
        } else {
            left_depth = 0;
        }
        if let Some(right) = &self.right {
            right_depth = 1 + right.calculate_depth();
        } else {
            right_depth = 0;
        }

        return max(left_depth, right_depth);
    }

    /// Walks through the tree along the path indicated by `path`, where `path` is interpreted as a
    /// binary value and every `0` bit is `left` and `1` is `right`. Walking starts at the root node,
    /// and at `path_length` number of bits from the least significant bit in the `path`.
    ///
    /// E.g: `path = 4` (binary `00...00100`) and `path_length = 4` would treat `path` as only `0100`.
    /// It will then walk through the tree, starting at the root node, and going `left -> right -> left -> left`.
    /// If it reaches a dead end, it will return the value of the node it is at, and the remainder
    /// of the `path_length` at that mode, such that `path_length - (y <- \result(x, y))` is the
    /// tree depth at the node that was returned. This is equivalent to the number of "consumed" bits.
    pub fn get_value_along_path(&self, path: u64, path_length: u32) -> Option<(&T, u32)> {
        if path_length == 0 {
            return if let Some(value) = &self.value {
                Some((value, 0))
            } else {
                None
            }
        }

        // If the bit we're interested in, is a 0, we go left. Else, it must be a 1 and we go right
        let go_left = (path >> (path_length - 1)) & 1u64 == 0;

        return if go_left {
            if let Some(left_subtree) = &self.left {
                left_subtree.get_value_along_path(path, path_length - 1)
            } else {
                if let Some(value) = &self.value {
                    Some((value, path_length))
                } else {
                    None
                }
            }
        } else {
            if let Some(right_subtree) = &self.right {
                right_subtree.get_value_along_path(path, path_length - 1)
            } else {
                if let Some(value) = &self.value {
                    Some((value, path_length))
                } else {
                    None
                }
            }
        }
    }
}

impl<T: Eq + Hash> HuffmanTree<T> {
    /// Generate an encoding map from this tree. The key of the map is the value in a node, the value
    /// is a tuple. The first element in the tuple is a binary representation of the path, where
    /// `left -> right -> left -> left` would be encoded as `0100`. The second value of the tuple is
    /// the length in bits. For the above example, the value would be `4`.
    /// Node values that appear more than once will only be represented in the map once
    pub fn generate_encoding_map(&self) -> HashMap<&T, (u32, u32)> {
        let all_values = self.get_all_values_with_paths(0, 0);

        let mut output_map = HashMap::new();
        for (k, v) in all_values {
            output_map.insert(k, v);
        }

        output_map
    }
}

impl<T: Display> HuffmanTree<T> {
    pub fn export_tree_dot_display(&self) -> String {
        return self.export_tree_dot(&|t| t.to_string());
    }
}

impl<T: Debug> HuffmanTree<T> {
    pub fn export_tree_dot_debug(&self) -> String {
        return self.export_tree_dot(&|t| format!("{:?}", t));
    }
}

impl<T: LowerHex> HuffmanTree<T> {
    pub fn export_tree_dot_hex(&self) -> String {
        return self.export_tree_dot(&|t| format!("0x{:x}", t));
    }
}

impl<T: Clone> HuffmanTree<T> {
    pub fn clone(&self) -> HuffmanTree<T> {
        let value = if let Some(v) = &self.value {
            Some((*v).clone())
        } else {
            None
        };

        let left = if let Some(l) = &self.left {
            Some(Box::new((*l).clone()))
        } else {
            None
        };

        let right = if let Some(r) = &self.right {
            Some(Box::new((*r).clone()))
        } else {
            None
        };

        HuffmanTree {
            value,
            left,
            right,
        }
    }
}

/// Inverts a binary number over the given length. This does not flip the individual bits, but rather
/// converts from big-endian to little-endian for a given length.
/// Helper function because it's easier to extract the paths from the tree in inverted order.
fn invert_binary(value: u32, length: u32) -> u32 {
    let mut output = 0u32;
    for index in 0..length {
        if (value & (1u32 << index)) > 0 {
            output += 2u32.pow(length - index - 1);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_set_path() {
        let mut tree = HuffmanTree::new();
        tree.set(5, 2, 3);

        let mut manual_tree = HuffmanTree {
            value: None,
            left: Some(Box::new(HuffmanTree {
                value: None,
                left: None,
                right: Some(Box::new(HuffmanTree {
                    value: None,
                    left: Some(Box::new(HuffmanTree {
                        value: Some(5),
                        left: None,
                        right: None,
                    })),
                    right: None,
                })),
            })),
            right: None,
        };

        assert_eq!(&manual_tree, &tree);

        tree.set(6, 3, 2);

        let manual_tree_part = HuffmanTree {
            value: None,
            left: None,
            right: Some(Box::new(HuffmanTree {
                value: Some(6),
                left: None,
                right: None,
            })),
        };
        manual_tree.right = Some(Box::new(manual_tree_part));

        assert_eq!(&manual_tree, &tree);
    }
}