use std::{
    env,
    fs::{create_dir_all, File},
    path::Path,
};

fn main() {
    let compression_level = 3;

    let resources = [
        (
            "FONT_REGULAR",
            "fonts/TeX-Gyre-Pagella/texgyrepagella-regular.otf",
        ),
        (
            "FONT_BOLD",
            "fonts/TeX-Gyre-Pagella/texgyrepagella-bold.otf",
        ),
        (
            "FONT_ITALIC",
            "fonts/TeX-Gyre-Pagella/texgyrepagella-italic.otf",
        ),
        (
            "FONT_BOLDITALIC",
            "fonts/TeX-Gyre-Pagella/texgyrepagella-bolditalic.otf",
        ),
        (
            "FONT_MATH",
            "fonts/TeX-Gyre-Pagella/texgyrepagella-math.otf",
        ),
        ("HYP_DE1996", "hyphenation-dicts/de-1996.standard.bincode"),
        ("HYP_EN_US", "hyphenation-dicts/en-us.standard.bincode"),
    ];

    let mut const_src_code = String::new();

    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    for (id, file) in resources {
        let out_dir = out_dir.join("compressed");

        let src_path = Path::new(file);
        let compressed_path = out_dir.join(file);

        create_dir_all(compressed_path.parent().unwrap()).unwrap();

        println!("cargo:rerun-if-changed={}", src_path.display());

        let font = File::open(src_path).unwrap();
        let font_compressed = File::create(compressed_path).unwrap();
        zstd::stream::copy_encode(font, font_compressed, compression_level).unwrap();

        // Write constant for the resource ID
        const_src_code.push_str(&format!("pub const {id}: &'static str = \"{file}\";\n"));
    }

    // Run macro to generate the get_decompress function for all resources
    const_src_code.push_str("embed_compressed!(\n");
    for (_id, file) in resources {
        const_src_code.push_str(&format!("\"{file}\",\n"));
    }
    const_src_code.push_str(");\n");

    std::fs::write(out_dir.join("resource_constants.rs"), const_src_code).unwrap();
}
