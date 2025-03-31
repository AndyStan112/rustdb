use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use bincode::{self, config, Decode, Encode};

const NODE_SIZE: usize = 1024;

#[derive(Encode, Decode, Debug)]
struct Node {
    key: Vec<u8>,
    offset: u64,
    left: Option<u64>,
    right: Option<u64>,
}

#[derive(Encode, Decode, Debug)]
struct IndexHeader {
    root_offset: Option<u64>,
    key_size: usize,
    t: usize,
}

fn write_node(file: &mut File, node: &Node) -> u64 {
    let mut buffer = [0u8; NODE_SIZE];
    let size = bincode::encode_into_slice(node, &mut buffer, config::standard()).expect("Failed to encode node");
    let offset = file.seek(SeekFrom::End(0)).expect("Failed to seek end of file");
    file.write_all(&buffer[..size]).expect("Failed to write node");
    if size < NODE_SIZE {
        file.write_all(&vec![0u8; NODE_SIZE - size]).expect("Failed to pad node");
    }
    offset
}

fn read_node(file: &mut File, offset: u64) -> Node {
    let mut buffer = vec![0u8; NODE_SIZE];
    file.seek(SeekFrom::Start(offset)).expect("Failed to seek to node offset");
    file.read_exact(&mut buffer).expect("Failed to read node");
    bincode::decode_from_slice(&buffer, config::standard()).expect("Failed to decode node").0
}

pub(crate) fn createindex(datafile: &str, recordsize: usize, keysize: usize) {
    let df = File::open(datafile).expect("Failed to open data file");
    let indexfile = format!("{}.ndx", datafile);
    let mut indexf = File::create(&indexfile).expect("Failed to create index file");

    let header = IndexHeader {
        root_offset: None,
        key_size: keysize,
        t: 2,
    };

    let mut header_buf = [0u8; 4096];
    let size = bincode::encode_into_slice(&header, &mut header_buf, config::standard()).expect("Failed to encode header");
    indexf.write_all(&header_buf[..size]).expect("Failed to write header");
    indexf.set_len(4096).expect("Failed to set header length");

    let mut offset = 0u64;
    let mut buffer = vec![0u8; recordsize];
    let mut df = df;
    while df.read_exact(&mut buffer).is_ok() {
        let key = buffer[..keysize].to_vec();
        insert_node(&indexfile, key, offset);
        offset += recordsize as u64;
    }
}

fn insert_node(indexfile: &str, key: Vec<u8>, data_offset: u64) {
    let mut file = OpenOptions::new().read(true).write(true).open(indexfile).expect("Failed to open index file");

    let mut header_buf = vec![0u8; 4096];
    file.seek(SeekFrom::Start(0)).expect("Failed to seek header");
    file.read_exact(&mut header_buf).expect("Failed to read header");
    let mut header: IndexHeader = bincode::decode_from_slice(&header_buf, config::standard()).expect("Failed to decode header").0;

    let new_node = Node {
        key: key.clone(),
        offset: data_offset,
        left: None,
        right: None,
    };

    let new_offset = write_node(&mut file, &new_node);

    if header.root_offset.is_none() {
        header.root_offset = Some(new_offset);
        file.seek(SeekFrom::Start(0)).expect("Failed to seek to header start");
        let mut header_out = [0u8; 4096];
        let size = bincode::encode_into_slice(&header, &mut header_out, config::standard()).expect("Failed to encode updated header");
        file.write_all(&header_out[..size]).expect("Failed to write updated header");
        return;
    }

    let mut current_offset = header.root_offset.unwrap();
    loop {
        let mut current = read_node(&mut file, current_offset);
        if key < current.key {
            if let Some(left_offset) = current.left {
                current_offset = left_offset;
            } else {
                current.left = Some(new_offset);
                file.seek(SeekFrom::Start(current_offset)).expect("Failed to seek to current node");
                let mut node_buf = [0u8; NODE_SIZE];
                let size = bincode::encode_into_slice(&current, &mut node_buf, config::standard()).expect("Failed to encode updated node");
                file.write_all(&node_buf[..size]).expect("Failed to write updated node");
                break;
            }
        } else {
            if let Some(right_offset) = current.right {
                current_offset = right_offset;
            } else {
                current.right = Some(new_offset);
                file.seek(SeekFrom::Start(current_offset)).expect("Failed to seek to current node");
                let mut node_buf = [0u8; NODE_SIZE];
                let size = bincode::encode_into_slice(&current, &mut node_buf, config::standard()).expect("Failed to encode updated node");
                file.write_all(&node_buf[..size]).expect("Failed to write updated node");
                break;
            }
        }
    }
}

pub(crate) fn searchrecord(datafile: &str, key: &[u8], recordsize: usize, keysize: usize) -> Option<Vec<u8>> {
    let indexfile = format!("{}.ndx", datafile);
    let mut file = File::open(&indexfile).ok()?;
    let mut buffer = vec![0u8; 4096];
    file.read_exact(&mut buffer).ok()?;
    let header: IndexHeader = bincode::decode_from_slice(&buffer, config::standard()).ok()?.0;

    let mut current_offset = header.root_offset?;
    while let node =read_node(&mut file, current_offset) {
        if key < &node.key {
            current_offset = node.left?;
        } else if key > &node.key {
            current_offset = node.right?;
        } else {
            let mut df = File::open(datafile).ok()?;
            df.seek(SeekFrom::Start(node.offset)).ok()?;
            let mut buf = vec![0u8; recordsize];
            df.read_exact(&mut buf).ok()?;
            return Some(buf[keysize..].to_vec());
        }
    }
    None
}

pub(crate) fn updaterecord(datafile: &str, key: &[u8], new_value: &[u8], keysize: usize) -> bool {
    let indexfile = format!("{}.ndx", datafile);
    let mut file = File::open(&indexfile).ok().unwrap();
    let mut buffer = vec![0u8; 4096];
    file.read_exact(&mut buffer).ok();
    let header: IndexHeader = bincode::decode_from_slice(&buffer, config::standard()).ok().unwrap().0;

    let Some(mut current_offset) = header.root_offset else {
        return false;
    };

    while let node = read_node(&mut file, current_offset) {
        if key < &node.key {
            current_offset = match node.left {
                Some(offset) => offset,
                None => return false,
            };
        } else if key > &node.key {
            current_offset = match node.right {
                Some(offset) => offset,
                None => return false,
            };
        } else {
            let mut df = OpenOptions::new().write(true).open(datafile).ok().unwrap();
            df.seek(SeekFrom::Start(node.offset + keysize as u64)).ok().unwrap();
            df.write_all(new_value).ok().unwrap();
            return true;
        }
    }
    false
}

pub(crate) fn addrecord(datafile: &str, key: &[u8], value: &[u8]) {
    let mut df = OpenOptions::new().append(true).open(datafile).expect("Failed to open data file for appending");
    let offset = df.seek(SeekFrom::End(0)).expect("Failed to seek end of data file");

    let mut record = key.to_vec();
    record.extend_from_slice(value);
    df.write_all(&record).expect("Failed to write record");

    insert_node(&format!("{}.ndx", datafile), key.to_vec(), offset);
}
