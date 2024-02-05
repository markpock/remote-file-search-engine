use std::{
    fs::File, io::{self, Seek, SeekFrom, Write}, mem::size_of
};
use crate::{memory_index::{MemIndex, DocTable}, utils::write_hashtable};

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
impl MemIndex {
    pub fn marshal(&self, name: &str) -> Result<(), io::Error> {
        let mut file = File::create(name)?;
        let canary = 0xDEADBEEFusize;
        file.write(canary.to_be_bytes().as_slice())?;
        
        let checksum: usize = 0xCAFECAFEusize;
        file.write(checksum.to_be_bytes().as_slice())?;

        self.write_to_file(&mut file)?;
        Ok(())
    }

    fn write_to_file(&self, file: &mut File) -> Result<usize, io::Error> {
        Ok (
            self.doctable.write_to_file(file)? +
            write_hashtable(file,
                &self.index,
                |f, k| {
                    Ok(f.write(k.len().to_be_bytes().as_slice())? + f.write(k.as_bytes())?)
                },
                |f, v| write_hashtable(
                    f, v,
                    |f_, k| f_.write(k.to_be_bytes().as_slice()),
                    |f_, v_| {
                        let mut accum = 0usize;
                        for &elt in v_ {
                            accum += f_.write(elt.to_be_bytes().as_slice())?;
                        }
                        Ok(accum)
                    }
                )
            )?
        )
    }
}

impl DocTable {
    pub(crate) fn write_to_file(&self, file: &mut File) -> Result<usize, io::Error> {
        // write offsets
        let original_position = file.stream_position()? as usize;
        let mut header_position = original_position;
        let mut vector_pos = original_position + 2 * size_of::<usize>();
        header_position += file.write(vector_pos.to_be_bytes().as_slice())?;
        file.seek(SeekFrom::Start(vector_pos as u64))?;
        // write id_to_doc
        let len = self.id_to_doc.len();
        vector_pos += file.write(len.to_be_bytes().as_slice())?;
        let mut element_pos = vector_pos + len * size_of::<usize>();
        for i in 0..len {
            vector_pos += file.write(element_pos.to_be_bytes().as_slice())?;
            file.seek(SeekFrom::Start(element_pos as u64))?;
            let str = self.id_to_doc[i].to_str().unwrap_or_else(|| "");
            element_pos += file.write(str.len().to_be_bytes().as_slice())?;
            element_pos += file.write(str.as_bytes())?;
        }
        file.seek(SeekFrom::Start(header_position as u64))?;

        // May need to add to header_position
        file.write(element_pos.to_be_bytes().as_slice())?;
        let ht_bytes = write_hashtable(
            file, 
            &self.doc_to_id, 
            |f, k| {
                let str = k.to_str().unwrap();
                Ok(f.write(str.len().to_be_bytes().as_slice())? + f.write(str.as_bytes())?)
            },
            |f, v| f.write(v.to_be_bytes().as_slice())
        )?;
        Ok(ht_bytes + element_pos - original_position)
    }
}
