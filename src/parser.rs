use std::{collections::HashMap, fs::File, io::{Read, Error}};

use crate::types::*;

pub fn parse_file(f: &mut File) -> Result<HashMap<String, Vec<Offset>>, Error> {
    let mut file_as_string = String::new();
    let _ = f.read_to_string(&mut file_as_string)?;

    let mut map: HashMap<String, Vec<Offset>> = HashMap::new();
    let mut intermediate: String = String::new();
    let mut beginning: Offset = 0;

    for (i, ch) in file_as_string.to_lowercase()
        .replace(|c: char| !c.is_alphabetic() && c != ' ' && c != '\n' && c != '\t', "")
        .chars()
        .enumerate()
    {
        if ch != ' ' && ch != '\t' && ch != '\n' { 
            intermediate.push(ch);
            continue;
        }
        if intermediate.len() > 0 {
            match map.get_mut(&intermediate) {
                Some(vec) => vec.push(beginning),
                None => { map.insert(intermediate.clone(), vec![beginning]); }
            }
        }
        beginning = i as Offset;
        intermediate.clear();
    }
    // Do this one more time, since there may not be whitespace at the end
    if intermediate.len() > 0 {
        match map.get_mut(&intermediate) {
            Some(vec) => vec.push(beginning),
            None => { map.insert(intermediate.clone(), vec![beginning]); }
        }
    }
    Ok(map)
}
