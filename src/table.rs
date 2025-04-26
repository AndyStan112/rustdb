use crate::btree::Index;
use std::path::Path;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

pub struct Table {
    pub keysize: u16,
    pub recordsize: u16,
    pub datafile: File,
    pub index: Index,
}

impl Table {
    pub fn create(path: &str, recordsize: u16, keysize: u16) -> io::Result<Self> {
        let indexfile = format!("{}.ndx", path);

        let entry_size = (keysize as u32) + 8 + (2 * (keysize as u32 + 8 + 8));

        //TODO recheck benchmarks because from my results 4 seems to be better everytime
        //But almost every major source I could find online uses delimeters similar to these
        let t = if entry_size <= 128 {
            32
        } else if entry_size <= 256 {
            16
        } else if entry_size <= 512 {
            8
        } else {
            4
        };

        Self::open_table(path, recordsize, keysize, &indexfile, t)
    }

    pub fn create_benchmark(path: &str, recordsize: u16, keysize: u16, indexfile: &str, t: u32) -> io::Result<Self> {
        Self::open_table(path, recordsize, keysize, indexfile, t)
    }

    fn open_table(datafile_path: &str, recordsize: u16, keysize: u16, indexfile: &str, t: u32) -> io::Result<Self> {
        let datafile = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(datafile_path)?;

        let index = if Path::new(indexfile).exists() {
            Index::open(indexfile)?
        } else {
            let mut idx = Index::create(indexfile, t, keysize)?;
            Self::create_index(datafile_path, keysize, recordsize, &mut idx)?;
            idx
        };

        Ok(Self {
            keysize,
            recordsize,
            datafile,
            index,
        })
    }


    fn create_index(path: &str, keysize: u16, recordsize: u16, index: &mut Index) -> io::Result<()> {
        let mut file = File::open(path)?;
        let entry_size = keysize as u64 + recordsize as u64;

        let mut offset = 0u64;
        let mut key_buf = vec![0u8; keysize as usize];

        loop {
            file.seek(SeekFrom::Start(offset))?;

            match file.read_exact(&mut key_buf) {
                Ok(_) => {
                    index.insert(key_buf.clone(), offset)?;
                    offset += entry_size;
                }
                Err(err) if err.kind() == io::ErrorKind::UnexpectedEof => {
                    break;
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }

        Ok(())
    }

    pub fn add_record(&mut self, key: &[u8], record: &[u8]) -> io::Result<()> {
        if key.len() > self.keysize as usize || record.len() > self.recordsize as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Key or record too large"));
        }

        if let Some(_) = self.index.search(key)? {
            println!("Warning: Key already exists. Use update_record instead.");
            return Ok(());
        }

        let offset = self.datafile.seek(SeekFrom::End(0))?;

        let mut fixed_key = vec![0u8; self.keysize as usize];
        fixed_key[..key.len()].copy_from_slice(key);
        self.datafile.write_all(&fixed_key)?;

        let mut fixed_record = vec![0u8; self.recordsize as usize];
        fixed_record[..record.len()].copy_from_slice(record);
        self.datafile.write_all(&fixed_record)?;

        self.datafile.flush()?;

        self.index.insert(fixed_key, offset)?;

        Ok(())
    }

    pub fn update_record(&mut self, key: &[u8], new_record: &[u8]) -> io::Result<()> {
        if key.len() > self.keysize as usize || new_record.len() > self.recordsize as usize {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "Key or record too large"));
        }

        if let Some(offset) = self.index.search(key)? {
            self.datafile.seek(SeekFrom::Start(offset))?;

            self.datafile.seek(SeekFrom::Current(self.keysize as i64))?;

            let mut fixed_record = vec![0u8; self.recordsize as usize];
            fixed_record[..new_record.len()].copy_from_slice(new_record);
            self.datafile.write_all(&fixed_record)?;
            self.datafile.flush()?;

            println!("Record for key updated successfully.");
            Ok(())
        } else {
            println!("Warning: Key not found. Cannot update non-existing record.");
            Ok(())
        }
    }


    pub fn search_record(&mut self, key: &[u8]) -> io::Result<Option<Vec<u8>>> {
        if let Some(offset) = self.index.search(key)? {
            self.datafile.seek(SeekFrom::Start(offset))?;

            let mut key_buf = vec![0u8; self.keysize as usize];
            self.datafile.read_exact(&mut key_buf)?;

            let mut record_buf = vec![0u8; self.recordsize as usize];
            self.datafile.read_exact(&mut record_buf)?;

            Ok(Some(record_buf))
        } else {
            Ok(None)
        }
    }

    pub fn list_records(&mut self) -> io::Result<()> {
        self.index.traverse_inorder(|key, offset| {
            self.datafile.seek(SeekFrom::Start(offset))?;

            let mut key_buf = vec![0u8; self.keysize as usize];
            self.datafile.read_exact(&mut key_buf)?;

            let mut record_buf = vec![0u8; self.recordsize as usize];
            self.datafile.read_exact(&mut record_buf)?;

            let key_str = String::from_utf8_lossy(key).trim_end_matches(char::from(0)).to_string();
            let record_str = String::from_utf8_lossy(&record_buf).trim_end_matches(char::from(0)).to_string();

            println!("Key: {}, Record: {}", key_str, record_str);

            Ok(())
        })
    }

}
