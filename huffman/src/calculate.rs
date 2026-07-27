use std::collections::{BinaryHeap, HashMap, HashSet};
use std::cmp::Ordering;

/// Return a map with each occurring chunk in data, and the number of times it appears.
/// Chunk size in bits must be passed in the `bits` parameter.
/// Returns `Err` if the bytes in `data` are not evenly divisible by the number of bits given,
/// or if `bits` is set to a value above `64`.
/// Setting `bits` to `8` will ensure this function always returns `Ok`.
pub fn count_chunks(data: &Vec<u8>, bits: u32) -> Result<HashMap<u64, u32>, String> {
    if bits > 64 {
        return Err(String::from("Cannot take chunks of more than 64 bits"));
    } else if (data.len() as u32 * 8) % bits != 0 {
        return Err(format!("Cannot divide the data into equal parts of {} bits", bits));
    }

    let mut buffer = 0u64;
    let mut buffer_size = 0;
    let mut map = HashMap::new();

    for byte in data {
        // Load into buffer
        buffer = buffer << 8; // Move the current contents over to make room for the new byte
        buffer = buffer | (*byte as u64); // BitWise OR it with the byte
        buffer_size += 8;

        // Retrieve from buffer
        while buffer_size >= bits { // If bits is less than 8, the buffer will fill faster
            // than it's emptied if we don't do a while loop here
            let next_value: u64 = buffer >> (buffer_size - bits); // Copy the buffer and move it so only the left-most bits-number of bits are left
            buffer_size -= bits;
            // Move 0xFFF..F over bits-number of bits so it becomes 1..1111110000 (if buffer_size = 4)
            // Invert that to 0..0001111, and BitWise AND it with the current buffer value to set the rest to 0
            buffer = buffer & !(u64::MAX << buffer_size);

            if let Some(count) = map.get_mut(&next_value) {
                *count += 1;
            } else {
                map.insert(next_value, 1);
            }
        }
    }

    Ok(map)
}

// Ordering of value set disregards whatever the set contains, and follows reverse ordering
#[derive(Debug)]
struct ValueSet {
    set: HashSet<u64>,
    value: u32,
}

impl PartialEq for ValueSet {
    fn eq(&self, other: &Self) -> bool {
        return self.value == other.value;
    }
}
impl Eq for ValueSet {}

impl PartialOrd for ValueSet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ValueSet {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.value < other.value {
            Ordering::Greater
        } else if self.value > other.value {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

// Benchmarks with flags -d -p -c 17 (24MB file)
// calculate_levels1: (sort every iteration)    -- 36.72 seconds
// calculate_levels2: (insert at correct place) -- 19.78 seconds
// calculate_levels3: (binary heap)             --  1.46 seconds

/// Calculate the levels at which each character will appear in the tree. This value is the same as
/// the number of bits each chunk will get in the encoding.
pub fn calculate_levels(counts: HashMap<u64, u32>) -> HashMap<u64, u32> {
    let mut levels = HashMap::new();

    let number_of_values = counts.len();
    let mut sets = BinaryHeap::with_capacity(number_of_values);

    // Add every unique chunk value to its own set, with its number of occurrences in the value
    for (chunk, count) in counts {
        sets.push(ValueSet {
            set: HashSet::from([chunk]),
            value: count,
        });
        levels.insert(chunk, 0);
    }

    // Run len once instead of every iteration
    let mut heap_size = number_of_values;

    while heap_size > 1 {
        let smallest_set = sets.pop().unwrap();
        let almost_smallest_set = sets.pop().unwrap();

        let total_count = smallest_set.value + almost_smallest_set.value;
        // Set union returns a HashSet<&u64> :(
        // So we do it manually
        let mut combo_set = HashSet::new();
        // But the values from the first set in the new one
        for e in smallest_set.set {
            combo_set.insert(e);
        }
        // And the values from the second set
        for e in almost_smallest_set.set {
            combo_set.insert(e);
        }
        // Add 1 to the level of every value that is now in this set
        for e in &combo_set {
            if let Some(v) = levels.get_mut(e) {
                *v += 1;
            }
        }

        sets.push(ValueSet {
            set: combo_set,
            value: total_count,
        });

        heap_size -= 1;
    }

    levels
}

/// Calculate the value of the binary number that can represent the path in the tree for each chunk
/// A value of 5 will become `101` or `left -> right -> left` in the tree (at level 3). This value
/// needs to be coupled with the chunk's level to actually calculate the path in the tree. In a tree
/// with a longest path of 8, our example from above would actually be 160 (`10100000`). Without the
/// level of `3`, the `101` could not be unambiguously retrieved.
pub fn calculate_path_values(levels: &HashMap<u64, u32>) -> HashMap<u64, u32> {
    // Get each chunk and its level in a vector, sorted by level
    let mut chunks: Vec<_> = levels.iter().map(|(c, l)| (*c, *l)).collect();
    chunks.sort_by(|(_va, la), (_vb, lb)| {
        return if la > lb {
            Ordering::Less
        } else if la == lb {
            Ordering::Equal
        } else {
            Ordering::Greater
        }
    });

    let highest_level = chunks.get(0).unwrap().1;

    let mut previous_value: u32 = 0;
    let mut path_values: HashMap<u64, u32> = HashMap::new();

    for (value, level) in chunks {
        path_values.insert(value, previous_value);
        previous_value = previous_value + 2i32.pow(highest_level - level) as u32;
    }

    return path_values;
}

