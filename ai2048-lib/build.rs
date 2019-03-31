use std::env;
use std::fs::File;
use std::io::Write;
use std::io;
use std::path::Path;

#[path = "src/build_common.rs"]
pub mod build_common;
use crate::build_common::{CACHE_SIZE, Row};

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_generated.rs");
    let mut f = File::create(&dest_path)?;

    f.write_all(b"const fn move_left_table() -> [Row; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Row(0); CACHE_SIZE];\n")?;
    for index in 0..CACHE_SIZE {
        let result = build_common::move_row_left(Row::from_index(index));
        f.write_all(format!("    cache[{}] = Row({});\n", index, result.0).as_str().as_bytes())?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn move_right_table() -> [Row; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Row(0); CACHE_SIZE];\n")?;
    for index in 0..CACHE_SIZE {
        let result = build_common::move_row_right(Row::from_index(index));
        f.write_all(format!("    cache[{}] = Row({});\n", index, result.0).as_str().as_bytes())?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn move_up_table() -> [Column; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Column(0); CACHE_SIZE];\n")?;
    for index in 0..CACHE_SIZE {
        let result = build_common::move_row_up(Row::from_index(index));
        f.write_all(format!("    cache[{}] = Column({});\n", index, result.0).as_str().as_bytes())?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn move_down_table() -> [Column; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Column(0); CACHE_SIZE];\n")?;
    for index in 0..CACHE_SIZE {
        let result = build_common::move_row_down(Row::from_index(index));
        f.write_all(format!("    cache[{}] = Column({});\n", index, result.0).as_str().as_bytes())?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    Ok(())
}