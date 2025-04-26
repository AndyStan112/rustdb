use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};

#[derive(Debug)]
struct Node {
    n: u32,
    keys: Vec<Vec<u8>>,
    values: Vec<u64>,
    children: Vec<i64>,
}

pub struct Index {
    file: File,
    t: u32,
    keysize: u16,
    pub(crate) root_offset: u64,
}

impl Index {
    pub fn create(path: &str, t: u32, keysize: u16) -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        // t (4) + root_offset (8) + keysize (2) = 14 reserved for header
        file.write_all(&[0u8; 14])?;

        let mut index = Index {
            file,
            t,
            keysize,
            root_offset: 0,
        };

        let root = Node {
            n: 0,
            keys: vec![],
            values: vec![],
            children: vec![-1; (2 * t) as usize],
        };

        index.root_offset = index.write_node(&root)?;
        index.write_header()?;

        Ok(index)
    }


    pub fn open(path: &str) -> io::Result<Self> {
        let mut file = OpenOptions::new().read(true).write(true).open(path)?;

        let mut buf4 = [0u8; 4];
        let mut buf8 = [0u8; 8];
        let mut buf2 = [0u8; 2];

        file.read_exact(&mut buf4)?;
        let t = u32::from_le_bytes(buf4);

        file.read_exact(&mut buf8)?;
        let root_offset = u64::from_le_bytes(buf8);

        file.read_exact(&mut buf2)?;
        let keysize = u16::from_le_bytes(buf2);

        Ok(Index { file, t, keysize, root_offset })
    }

    fn write_header(&mut self) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(0))?;
        self.file.write_all(&self.t.to_le_bytes())?;
        self.file.write_all(&self.root_offset.to_le_bytes())?;
        self.file.write_all(&self.keysize.to_le_bytes())?;
        Ok(())
    }

    fn serialize_node(&mut self, node: &Node) -> io::Result<()> {
        self.file.write_all(&node.n.to_le_bytes())?;

        let max_keys = (2 * self.t - 1) as usize;
        for i in 0..max_keys {
            if i < node.keys.len() {
                let key = &node.keys[i];
                let mut fixed_key = vec![0u8; self.keysize as usize];
                let len = key.len().min(self.keysize as usize);
                fixed_key[..len].copy_from_slice(&key[..len]);
                self.file.write_all(&fixed_key)?;
                self.file.write_all(&node.values[i].to_le_bytes())?;
            } else {
                self.file.write_all(&vec![0u8; self.keysize as usize])?;
                self.file.write_all(&0u64.to_le_bytes())?;
            }
        }

        let max_children = (2 * self.t) as usize;
        for i in 0..max_children {
            if i < node.children.len() {
                self.file.write_all(&node.children[i].to_le_bytes())?;
            } else {
                self.file.write_all(&(-1i64).to_le_bytes())?;
            }
        }

        Ok(())
    }

    fn write_node(&mut self, node: &Node) -> io::Result<u64> {
        let offset = self.file.seek(SeekFrom::End(0))?;
        self.serialize_node(node)?;
        Ok(offset)
    }

    fn write_node_at(&mut self, offset: u64, node: &Node) -> io::Result<()> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.serialize_node(node)
    }


    fn read_node(&mut self, offset: u64) -> io::Result<Node> {
        self.file.seek(SeekFrom::Start(offset))?;

        let mut buf4 = [0u8; 4];
        self.file.read_exact(&mut buf4).expect("failed read");
        let n = u32::from_le_bytes(buf4);

        let max_keys = (2 * self.t - 1) as usize;
        let mut keys = Vec::new();
        let mut values = Vec::new();

        for _ in 0..max_keys {
            let mut key_buf = vec![0u8; self.keysize as usize];
            self.file.read_exact(&mut key_buf)?;
            keys.push(key_buf);

            let mut value_buf = [0u8; 8];
            self.file.read_exact(&mut value_buf)?;
            values.push(u64::from_le_bytes(value_buf));
        }

        let max_children = (2 * self.t) as usize;
        let mut children = Vec::new();
        for _ in 0..max_children {
            let mut child_buf = [0u8; 8];
            self.file.read_exact(&mut child_buf)?;
            children.push(i64::from_le_bytes(child_buf));
        }

        Ok(Node { n, keys, values, children })
    }

    pub fn insert(&mut self, key: Vec<u8>, value: u64) -> io::Result<()> {
        let root = self.read_node(self.root_offset)?;
        if root.n as usize == (2 * self.t - 1) as usize {
            let mut new_root = Node {
                n: 0,
                keys: vec![],
                values: vec![],
                children: vec![-1; (2 * self.t) as usize],
            };

            let old_root_offset = self.root_offset;
            let new_root_offset = self.write_node(&new_root)?;
            self.root_offset = new_root_offset;
            self.write_header()?;

            new_root.children[0] = old_root_offset as i64;
            self.split_child(&mut new_root, 0, old_root_offset)?;

            self.write_node_at(new_root_offset, &new_root)?;

            self.insert_non_full(self.root_offset, key, value)
        } else {
            self.insert_non_full(self.root_offset, key, value)
        }
    }

    fn insert_non_full(&mut self, offset: u64, key: Vec<u8>, value: u64) -> io::Result<()> {
        let mut node = self.read_node(offset)?;

        let mut i = node.n as isize - 1;
        if node.children[0] == -1 {
            while i >= 0 && node.keys[i as usize] > key {
                i -= 1;
            }
            node.keys.insert((i + 1) as usize, key);
            node.values.insert((i + 1) as usize, value);
            node.n += 1;

            self.write_node_at(offset, &node)?;
            Ok(())
        } else {
            while i >= 0 && node.keys[i as usize] > key {
                i -= 1;
            }
            i += 1;
            let child_offset = node.children[i as usize] as u64;
            let child = self.read_node(child_offset)?;

            if child.n as usize == (2 * self.t - 1) as usize {
                self.split_child(&mut node, i as usize, child_offset)?;

                if node.keys[i as usize] < key {
                    i += 1;
                }
            }

            self.write_node_at(offset, &node)?;

            self.insert_non_full(node.children[i as usize] as u64, key, value)
        }
    }

    fn split_child(&mut self, parent: &mut Node, i: usize, child_offset: u64) -> io::Result<()> {
        let mut y = self.read_node(child_offset)?;

        let mut z = Node {
            n: self.t - 1,
            keys: Vec::new(),
            values: Vec::new(),
            children: vec![-1; (2 * self.t) as usize],
        };

        for _ in 0..(self.t - 1) {
            z.keys.push(y.keys.remove(self.t as usize));
            z.values.push(y.values.remove(self.t as usize));
        }

        if y.children[0] != -1 {
            for j in 0..self.t as usize {
                z.children[j] = y.children.remove(self.t as usize);
            }
        }

        y.n = self.t - 1;

        let z_offset = self.write_node(&z)?;

        parent.keys.insert(i, y.keys.remove((self.t - 1) as usize));
        parent.values.insert(i, y.values.remove((self.t - 1) as usize));
        parent.children.insert(i + 1, z_offset as i64);
        parent.n += 1;

        self.write_node_at(child_offset, &y)?;

        Ok(())
    }


    pub fn traverse_inorder_from<F>(&mut self, offset: u64, mut visit: F) -> io::Result<()>
    where
        F: FnMut(&[u8], u64) -> io::Result<()>,
    {
        let node = self.read_node(offset)?;

        for i in 0..(node.n as usize) {
            if node.children[i] != -1 {
                self.traverse_inorder_from(node.children[i] as u64, &mut visit)?;
            }

            visit(node.keys[i].as_slice(), node.values[i])?;
        }

        if node.children[node.n as usize] != -1 {
            self.traverse_inorder_from(node.children[node.n as usize] as u64, &mut visit)?;
        }

        Ok(())
    }

    pub fn traverse_inorder<F>(&mut self, mut visit: F) -> io::Result<()>
    where
        F: FnMut(&[u8], u64) -> io::Result<()>,
    {
        self.traverse_inorder_from(self.root_offset, visit)
    }

    pub fn search(&mut self, key: &[u8]) -> io::Result<Option<u64>> {
        self.search_in_node(self.root_offset, key)
    }

    fn search_in_node(&mut self, offset: u64, key: &[u8]) -> io::Result<Option<u64>> {
        let node = self.read_node(offset)?;

        let mut fixed_key = vec![0u8; self.keysize as usize];
        let len = key.len().min(self.keysize as usize);
        fixed_key[..len].copy_from_slice(&key[..len]);

        let mut low = 0;
        let mut high = node.n as usize;

        while low < high {
            let mid = (low + high) / 2;
            if node.keys[mid].as_slice() < fixed_key.as_slice() {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        if low < node.n as usize && node.keys[low].as_slice() == fixed_key.as_slice() {
            return Ok(Some(node.values[low]));
        }

        if node.children[low] == -1 {
            Ok(None)
        } else {
            self.search_in_node(node.children[low] as u64, key)
        }
    }


}
