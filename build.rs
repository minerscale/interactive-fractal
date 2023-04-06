// build.rs

use std::env;
use std::fs;
use std::io::BufWriter;
use std::path::Path;

use arbitrary_fixed::ArbitraryFixed;
use arbitrary_fixed_glsl::write_const;

fn main() -> std::io::Result<()> {
    let in_str = fs::read_to_string("src/cs.rs")?;
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("cs.rs");
    fs::write(
        &dest_path,
        in_str.replace(
            "INCLUDE_PATH",
            &format!(
                "\"{}\", \"{}\"",
                arbitrary_fixed_glsl::include_dir(),
                out_dir.to_str().unwrap()
            ),
        ),
    )?;
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src/cs.rs");

    let const_path = Path::new(&out_dir).join("local_consts.glsl");

    let mut f = BufWriter::new(fs::File::create(const_path)?);

    write_const(&mut f, "FIX_SIXTY_FOUR", ArbitraryFixed::from(64u32))?;
    write_const(&mut f, "FIX_NEG_HALF", -ArbitraryFixed::from(1u32).rshift1())?;

    Ok(())
}
