mod btree;
mod table;

use table::Table;
use std::fs::{OpenOptions, remove_file};
use std::io::{self, Write};
use std::path::Path;
use std::time::Instant;

struct BenchmarkResult {
    file: String,
    t: u32,
    load_time: f64,
    search_time: f64,
    add_time: f64,
    update_time: f64,
    search_after_update_time: f64,
}

fn benchmark_table(
    datafile: &str,
    recordsize: u16,
    keysize: u16,
    sample_key: &[u8],
    add_key: &[u8],
    add_record: &[u8],
    update_record: &[u8],
    t: u32,
) -> io::Result<BenchmarkResult> {
    let static_path = format!("static/{}", datafile);

    println!("Benchmarking file: {} with t = {}\n", static_path, t);

    let indexfile = format!("static/{}.t{}.ndx", datafile, t);
    if Path::new(&indexfile).exists() {
        remove_file(&indexfile)?;
    }

    let tmp_datafile = format!("static/{}.t{}.dat", datafile, t);
    std::fs::copy(&static_path, &tmp_datafile)?;

    let start = Instant::now();
    let mut table = Table::create_benchmark(&tmp_datafile, recordsize, keysize, &indexfile, t)?;
    let load_duration = start.elapsed();
    println!("Load/Create Table: {:.4?}", load_duration);

    let start = Instant::now();
    match table.search_record(sample_key)? {
        Some(_) => println!("Search for key {:?}: Found", sample_key),
        None => println!("Search for key {:?}: Not found", sample_key),
    }
    let search_duration = start.elapsed();
    println!("Search Time: {:.4?}", search_duration);

    let start = Instant::now();
    table.add_record(add_key, add_record)?;
    let add_duration = start.elapsed();
    println!("Add Record Time: {:.4?}", add_duration);

    let start = Instant::now();
    table.update_record(add_key, update_record)?;
    let update_duration = start.elapsed();
    println!("Update Record Time: {:.4?}", update_duration);

    let start = Instant::now();
    let found = table.search_record(add_key)?;
    let search_after_update_duration = start.elapsed();

    if let Some(val) = found {
        let trimmed_val = &val[..update_record.len()];
        if trimmed_val != update_record {
            panic!(
                "Error: Updated record does not match expected!\nExpected: \n{:?} got\n{:?}",
                update_record, trimmed_val
            );
        }
    } else {
        panic!("Error: Updated key not found!");
    }

    println!("Search after update Time: {:.4?}", search_after_update_duration);
    println!("----------------------------------------\n");

    remove_file(&tmp_datafile)?;

    Ok(BenchmarkResult {
        file: datafile.to_string(),
        t,
        load_time: load_duration.as_secs_f64(),
        search_time: search_duration.as_secs_f64(),
        add_time: add_duration.as_secs_f64(),
        update_time: update_duration.as_secs_f64(),
        search_after_update_time: search_after_update_duration.as_secs_f64(),
    })
}

fn main() -> io::Result<()> {
    let mut results_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("static/results.csv")?;

    writeln!(
        results_file,
        "file,t,load_time,search_time,add_time,update_time,search_after_update_time"
    )?;

    let t_values = [2, 4, 8, 16];

    let mut results = Vec::new();

    for &t in &t_values {
        results.push(benchmark_table(
            "small.dat",
            31,
            1,
            b"G",
            b"Z",
            b"new small record ............",
            b"updated small record ........",
            t,
        )?);
    }

    for &t in &t_values {
        results.push(benchmark_table(
            "medium.dat",
            80,
            4,
            b"2499",
            b"9999",
            b"new medium record ....................data here...................",
            b"updated medium record .................new data here...............",
            t,
        )?);
    }

    for &t in &t_values {
        results.push(benchmark_table(
            "large.dat",
            102,
            7,
            b"1000942",
            b"9999999",
            b"new large record .....................................................extra data here..........",
            b"updated large record ..................................................updated extra data.......",
            t,
        )?);
    }

    for r in results.iter() {
        writeln!(
            results_file,
            "{},{},{:.6},{:.6},{:.6},{:.6},{:.6}",
            r.file,
            r.t,
            r.load_time,
            r.search_time,
            r.add_time,
            r.update_time,
            r.search_after_update_time
        )?;
    }

    println!("Benchmark finished. Results saved to static/results.csv!");

    println!("\nSummary:");
    println!("{:<12} {:<4} {:<10} {:<10} {:<10} {:<10} {:<10}",
             "File", "t", "Load(s)", "Search(s)", "Add(s)", "Update(s)", "Search2(s)");

    for r in &results {
        println!("{:<12} {:<4} {:<10.6} {:<10.6} {:<10.6} {:<10.6} {:<10.6}",
                 r.file,
                 r.t,
                 r.load_time,
                 r.search_time,
                 r.add_time,
                 r.update_time,
                 r.search_after_update_time,
        );
    }

    Ok(())
}
