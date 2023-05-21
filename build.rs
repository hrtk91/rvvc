fn main() {
    use fs_extra::dir::CopyOptions;
    use std::env;
    use std::path::{Path, PathBuf};

    // ビルド後の出力先フォルダのパスを取得
    let out_dir = env::var("OUT_DIR").unwrap();
    let out_dir = dunce::canonicalize(Path::new(&out_dir).join("../../../")).unwrap();

    println!("out_dir          {}", out_dir.to_str().unwrap());

    let voicevox_dir = dunce::canonicalize("voicevox_core").unwrap();
    let voicevox_dst_dir = out_dir.join("voicevox_core");
    println!("voicevox_dir     {}", voicevox_dir.to_str().unwrap());
    println!("voicevox_dst_dir {}", voicevox_dst_dir.to_str().unwrap());

    let options = CopyOptions::new().overwrite(true);

    let targets: Vec<PathBuf> = std::fs::read_dir("voicevox_core")
        .unwrap()
        .map(|x| x.unwrap().path())
        .collect();

    fs_extra::copy_items(&targets, out_dir.as_path(), &options)
        .expect("failed to copy voicevox_core");

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.to_str().expect("failed to get out_dir")
    );
    println!("cargo:rustc-link-lib=dylib=voicevox_core");

    // // Cのヘッダーファイルのパス
    let header_path = "voicevox_core/voicevox_core.h";

    // バインディングのオプションを設定
    let bindings = bindgen::Builder::default()
        .header(header_path)
        .generate()
        .expect("Failed to generate bindings");

    // bindingsの先頭行に`!#[allow(warnings)]`を追記
    let bindings = format!("#![allow(warnings)]\r\n{}", bindings.to_string());

    // バインディングをRustのソースファイルに書き出す
    std::fs::write("src/bindings.rs", &bindings).expect("Failed to write bindings");
}
