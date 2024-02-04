use std::{
    collections::{hash_map::DefaultHasher, HashMap, LinkedList},
    io::{self, Seek, SeekFrom, Write},
    hash::{Hash, Hasher},
    mem::size_of,
    fs::File
};
use crate::memindex::MemIndex;

/**
 * Memory layout:
 * CANARY
 * CHECKSUM
 * DOCTABLE
 *      offset of vector
 *      offset of hashmap
 *      length
 *      list of absolute offsets
 *          absolute offset 0
 *          absolute offset 1
 *      docs at each absolute offset
 *          @ abs offset 0, doc 0
 *          @ abs offset 1, doc 1
 *      number of buckets (capacity of doctable)
 *      list of bucket offsets and lengths
 *          absolute bucket offset 0
 *          number of elements in chain at bucket 0
 *          ...
 *      buckets
 *          bucket 0
 *              list of absolute offsets
 *                  absolute offset of doc n
 *                  absolute offset of doc m
 *              id n
 *              len of doc n
 *              doc n      
 *              id m
 *              len of doc m
 *              doc m
 *          bucket 1
 *              doc l
 *              id l
 *          ...
 * MEMINDEX
 *      number of buckets (capacity of memindex)
 *      list of bucket offsets
 *          absolute bucket offset 0; length of chain
 *          ...
 *      buckets
 *          bucket 0
 *              doc n
 *              id n
 *              doc m
 *              id m
 *          bucket 2
 *              doc l
 *              id l
 *          ...
 *  
 * 
 * In general, a hashtable is:
 * {
 *      capacity: usize
 *      buckrecs: Vec (offset : usize, chainlength : usize) capacity
 *      buckets: Vec (bucket _) capacity with (bucket i) = {
 *          Vec usize (buckrecs i).chainlength
 *          Vec (element _ ) (buckrecs i).chainlength with (element j) = {
 *              len key: usize
 *              len value: usize
 *              key of len key bytes
 *              value of len value bytes
 *          }
 *      }
 * }
 */

pub fn marshal(name: &str, memindex: MemIndex) -> Result<(), io::Error> {
    let mut file = File::create(name)?;
    let canary = 0xDEADBEEFusize;
    file.write(canary.to_be_bytes().as_slice())?;
    
    let checksum: usize = 0xCAFECAFEusize;
    file.write(checksum.to_be_bytes().as_slice())?;

    memindex.write_to_file(&mut file)?;
    Ok(())
}

pub fn write_hashtable<K: Hash, V, FK, FV>(
    file: &mut File,
    ht: &HashMap<K, V>,
    write_key: FK,
    write_value: FV
) -> Result<usize, io::Error>
where FK: Fn(&mut File, &K) -> Result<usize, io::Error>,
    FV: Fn(&mut File, &V) -> Result<usize, io::Error>
{
    let mut vec: Vec<LinkedList<(&K, &V)>> = Vec::with_capacity(ht.capacity());
    let len = vec.capacity();
    for _ in 0..len {
        vec.push(LinkedList::new())
    }

    for (key, value) in ht {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        vec[hasher.finish() as usize % len].push_back((key, value));
    }

    // All of these offsets are absolute
    let original_position = file.stream_position()? as usize;
    let mut table_header_offset = original_position;
    table_header_offset += file.write(len.to_be_bytes().as_slice())?;

    let mut bucket_offset = table_header_offset + 2 * size_of::<usize>() * len;
    for bucket in 0..len {
        table_header_offset += file.write(bucket_offset.to_be_bytes().as_slice())?;
        table_header_offset += file.write(vec[bucket].len().to_be_bytes().as_slice())?;

        let mut bucket_header_offset = file.seek(SeekFrom::Start(bucket_offset as u64))? as usize;
        let mut element_offset = size_of::<usize>() * vec[bucket].len() + bucket_header_offset;
        for (k, v) in vec[bucket].iter() {
            bucket_header_offset += file.write(element_offset.to_be_bytes().as_slice())?;
            element_offset = file.seek(SeekFrom::Start(element_offset as u64))? as usize;

            let mut element_header_offset = element_offset;

            file.seek(SeekFrom::Current(2 * size_of::<usize>() as i64))?;
            element_offset += 2 * size_of::<usize>();

            let key_bytes = write_key(file, k)? as usize;
            element_offset += key_bytes;
            file.seek(SeekFrom::Start(element_header_offset as u64))?;
            element_header_offset += file.write(key_bytes.to_be_bytes().as_slice())?;

            file.seek(SeekFrom::Start(element_offset as u64))?;
            let value_bytes = write_value(file, v)? as usize;
        
            file.seek(SeekFrom::Start(element_header_offset as u64))?;
            element_header_offset += file.write(value_bytes.to_be_bytes().as_slice())?;
            element_offset += value_bytes;

            bucket_header_offset = file.seek(SeekFrom::Start(bucket_header_offset as u64))? as usize;
        }
        table_header_offset = file.seek(SeekFrom::Start(table_header_offset as u64))? as usize;
        bucket_offset = element_offset;
    }
    Ok (bucket_offset - original_position)
}
