use std::{
    fs::{File, OpenOptions}, io::{self, Read, Seek, SeekFrom, Write}, mem::size_of
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
        // let mut file = File::create(name)?;
        let mut file = OpenOptions::new().read(true).write(true).open(name)?;
        let canary = 0xDEADBEEFusize;
        file.write(canary.to_be_bytes().as_slice())?;
        
        let checksum: usize = 0xCAFECAFEusize;
        file.write(checksum.to_be_bytes().as_slice())?;

        self.write_to_file(&mut file)?;
        file.flush()?;
        Ok(())
    }

    fn write_to_file(&self, file: &mut File) -> Result<usize, io::Error> {
        let mut original = file.stream_position()? as usize;
        file.seek(SeekFrom::Current(2 * size_of::<usize>() as i64))?;
        let doctable_bytes = self.doctable.write_to_file(file)?;

        println!("current memindex pos {}", file.stream_position()? as usize);

        file.seek(SeekFrom::Start(original as u64))?;
        println!("position of doctable bytes {}", original);
        println!("doctable bytes {}", doctable_bytes);

        original += file.write(doctable_bytes.to_be_bytes().as_slice())?;
        file.seek(SeekFrom::Current((size_of::<usize>() + doctable_bytes) as i64))?;

        println!("actual memindex pos {}", file.stream_position()? as usize);

        let memindex_bytes = write_hashtable(file,
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
        )?;
        file.seek(SeekFrom::Start(original as u64))?;
        println!("position of memindex bytes {}", original);
        println!("memindex bytes {}", memindex_bytes);

        file.write(memindex_bytes.to_be_bytes().as_slice())?;
        Ok(2 * size_of::<usize>() + doctable_bytes + memindex_bytes)
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

        println!("offset of veclen: {}", file.stream_position()?);

        vector_pos += file.write(len.to_be_bytes().as_slice())?;

        println!("len at marshalled: {}", len);

        let mut element_pos = vector_pos + len * size_of::<usize>();
        for i in 0..len {
            vector_pos += file.write(element_pos.to_be_bytes().as_slice())?;
            file.seek(SeekFrom::Start(element_pos as u64))?;
            let str = self.id_to_doc[i].to_str().unwrap_or_else(|| "");
            let strlen = str.len();
            element_pos += file.write(strlen.to_be_bytes().as_slice())?;
            element_pos += file.write(str.as_bytes())?;
            element_pos += 1;
            element_pos += 8 - (element_pos % 8);
            file.seek(SeekFrom::Start(vector_pos as u64))?;
        }
        file.seek(SeekFrom::Start(header_position as u64))?;

        println!("What");

        // May need to add to header_position
        file.write(element_pos.to_be_bytes().as_slice())?;

        file.seek(SeekFrom::Start(element_pos as u64))?;
        let ht_bytes = write_hashtable(
            file, 
            &self.doc_to_id, 
            |f, k| {
                let str = k.to_str().unwrap();
                Ok(f.write(str.len().to_be_bytes().as_slice())? + f.write(str.as_bytes())?)
            },
            |f, v| f.write(v.to_be_bytes().as_slice())
        )?;

        println!("What2");

        file.seek(SeekFrom::Start(48))?;

        println!("What1");

        let buf = 0usize;
        let mut geh = usize::to_be_bytes(buf);

        println!("What4");
        file.read_exact(geh.as_mut_slice())?;
        println!("What3");
        println!("current veclen {}", usize::from_be_bytes(geh));

        file.seek(SeekFrom::Start((original_position + ht_bytes + (element_pos - original_position)) as u64))?;
        Ok(ht_bytes + (element_pos - original_position))
    }
}
