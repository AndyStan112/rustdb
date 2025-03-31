mod btree;

fn main() {
    btree::createindex("small.dat", 31, 1);
    btree::addrecord("small.dat", b"X", b"this is a test record.............\n");
    let value = btree::searchrecord("small.dat", b"X", 31, 1).unwrap();
    let s = std::str::from_utf8(&*value).expect("invalid utf-8 sequence");
    println!("{}", s);

    btree::updaterecord("small.dat", b"X", b"updated record.................", 1);
    let value = btree::searchrecord("small.dat", b"X", 31, 1).unwrap();
    let s = std::str::from_utf8(&*value).expect("invalid utf-8 sequence");
    println!("{}", s);
}
