use std::{
    env,
    fs::{create_dir_all, File},
    path::Path,
};

fn main() {
    let fonts = [
        "TeX-Gyre-Pagella/texgyrepagella-regular.otf",
        "TeX-Gyre-Pagella/texgyrepagella-bold.otf",
        "TeX-Gyre-Pagella/texgyrepagella-italic.otf",
        "TeX-Gyre-Pagella/texgyrepagella-bolditalic.otf",
        "TeX-Gyre-Pagella/texgyrepagella-math.otf",
    ];

    for font in fonts {
        let out_dir = env::var("OUT_DIR").unwrap();
        let font_out_dir = Path::new(&out_dir).join("fonts/compressed");

        let font_path = Path::new("fonts").join(font);
        let font_compressed_path = font_out_dir.join(font);

        create_dir_all(font_compressed_path.parent().unwrap()).unwrap();

        println!("cargo:rerun-if-changed={}", font_path.display());

        let font = File::open(font_path).unwrap();
        let font_compressed = File::create(font_compressed_path).unwrap();
        zstd::stream::copy_encode(font, font_compressed, zstd::DEFAULT_COMPRESSION_LEVEL).unwrap();
    }
}
