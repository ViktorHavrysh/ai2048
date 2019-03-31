use std::env;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::Path;

#[path = "src/build_common.rs"]
pub mod build_common;

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("build_generated.rs");
    let mut f = File::create(&dest_path)?;

    f.write_all(b"const fn move_left_table() -> [Row; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Row(0); CACHE_SIZE];\n")?;
    for (index, row) in build_common::all_rows() {
        let result = build_common::move_row_left(row);
        f.write_all(
            format!("    cache[{}] = Row({});\n", index, result.0)
                .as_str()
                .as_bytes(),
        )?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn move_right_table() -> [Row; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Row(0); CACHE_SIZE];\n")?;
    for (index, row) in build_common::all_rows() {
        let result = build_common::move_row_right(row);
        f.write_all(
            format!("    cache[{}] = Row({});\n", index, result.0)
                .as_str()
                .as_bytes(),
        )?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn move_up_table() -> [Column; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Column(0); CACHE_SIZE];\n")?;
    for (index, row) in build_common::all_rows() {
        let result = build_common::move_row_up(row);
        f.write_all(
            format!("    cache[{}] = Column({});\n", index, result.0)
                .as_str()
                .as_bytes(),
        )?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn move_down_table() -> [Column; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [Column(0); CACHE_SIZE];\n")?;
    for (index, row) in build_common::all_rows() {
        let result = build_common::move_row_down(row);
        f.write_all(
            format!("    cache[{}] = Column({});\n", index, result.0)
                .as_str()
                .as_bytes(),
        )?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    f.write_all(b"const fn heur_table() -> [f32; CACHE_SIZE] {\n")?;
    f.write_all(b"    let mut cache = [0.0f32; CACHE_SIZE];\n")?;
    for (index, row) in build_common::all_rows() {
        let result = build_common::eval_row(row);
        f.write_all(
            format!("    cache[{}] = {} as f32;\n", index, result)
                .as_str()
                .as_bytes(),
        )?;
    }
    f.write_all(b"    cache\n")?;
    f.write_all(b"}\n")?;

    Ok(())
}
