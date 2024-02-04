use std::{collections::{hash_map::DefaultHasher, HashMap, LinkedList}, fs::File, hash::{Hash, Hasher}, io::{self, Seek, SeekFrom, Write}, mem::size_of};

pub type DocID = u32;
pub type Offset = u64;

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

            // May need to add to element header offset
            file.write(value_bytes.to_be_bytes().as_slice())?;
            element_offset += value_bytes;

            bucket_header_offset = file.seek(SeekFrom::Start(bucket_header_offset as u64))? as usize;
        }
        table_header_offset = file.seek(SeekFrom::Start(table_header_offset as u64))? as usize;
        bucket_offset = element_offset;
    }
    Ok (bucket_offset - original_position)
}
