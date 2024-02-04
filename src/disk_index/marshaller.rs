use std::{
    io::{self, Write},
    fs::File
};
use crate::memory_index::MemIndex;

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
