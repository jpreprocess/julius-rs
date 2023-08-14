use std::{
    error::Error,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

fn main() -> Result<(), Box<dyn Error>> {
    let build_dir_str = std::env::var_os("OUT_DIR").unwrap();
    let build_dir = Path::new(&build_dir_str);

    let julius_dir = prepare_source(build_dir)?;

    eprintln!("--- libsent/configure patch start ---");

    let mut patch = Command::new("patch")
        .arg("-p1")
        .arg("libsent/configure")
        .current_dir(&julius_dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .spawn()?;
    {
        let mut patchstdin = patch.stdin.as_ref().unwrap();
        let mut writer = BufWriter::new(&mut patchstdin);
        writer.write_all(&std::fs::read("configure.patch")?)?;
    }
    patch.wait()?;

    eprintln!("--- libsent/configure patch end ---");

    eprintln!("--- configure start ---");

    let configure_output = Command::new("./configure")
        .arg("--enable-words-int")
        .arg("--with-mictype=none")
        .current_dir(&julius_dir)
        .output()?;

    eprintln!("{}\n", std::str::from_utf8(&configure_output.stdout)?);
    eprintln!("{}\n", std::str::from_utf8(&configure_output.stderr)?);

    eprintln!("--- configure end ---");

    eprintln!("--- make start ---");

    let make_output = Command::new("make").current_dir(&julius_dir).output()?;
    eprintln!("{}\n", std::str::from_utf8(&make_output.stdout)?);
    eprintln!("{}\n", std::str::from_utf8(&make_output.stderr)?);

    eprintln!("--- make end ---");

    println!(
        "cargo:rustc-link-search={}",
        julius_dir.join("libjulius").to_str().unwrap()
    );
    println!(
        "cargo:rustc-link-search={}",
        julius_dir.join("libsent").to_str().unwrap()
    );

    println!("cargo:rustc-link-lib=static=julius");
    println!("cargo:rustc-link-lib=static=sent");
    println!("cargo:rustc-link-lib=dylib=z");
    println!("cargo:rustc-link-lib=dylib=gomp");

    generate_bindings(julius_dir.as_path());

    Ok(())
}

#[cfg(not(feature = "generate-bindings"))]
fn generate_bindings(_julius_dir: &Path) {}

#[cfg(feature = "generate-bindings")]
fn generate_bindings(julius_dir: &Path) {
    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .clang_args([
            format!(
                "-F{}",
                julius_dir.join("libjulius/include").to_str().unwrap()
            ),
            format!("-F{}", julius_dir.join("libsent/include").to_str().unwrap()),
        ])
        .allowlist_file(format!("{}.*", julius_dir.to_str().unwrap()))
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from("src");
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    println!("cargo:rerun-if-changed=wrapper.h");
}

fn prepare_source(build_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    use flate2::read::GzDecoder;
    use std::io::Read;
    use tar::Archive;

    let file_name = "v4.6.tar.gz";

    // Source file path for build package
    let source_path_for_build = build_dir.join(file_name);

    // Download source file to build directory
    if !source_path_for_build.exists() {
        // copy(&source_path, &source_path_for_build)?;
        let tmp_path = build_dir.join(file_name.to_owned() + ".download");

        // Download a tarball
        let download_url = "https://github.com/julius-speech/julius/archive/refs/tags/v4.6.tar.gz";
        let resp = ureq::get(download_url).call()?;
        let mut dest = File::create(&tmp_path)?;

        std::io::copy(&mut resp.into_reader(), &mut dest)?;
        dest.flush()?;

        std::fs::rename(tmp_path, &source_path_for_build).expect("Failed to rename temporary file");
    }

    // Decompress a tar.gz file
    let mut tar_gz = File::open(source_path_for_build)?;
    let mut buffer = Vec::new();
    tar_gz.read_to_end(&mut buffer)?;
    let cursor = std::io::Cursor::new(buffer);
    let gzdecoder = GzDecoder::new(cursor);
    let mut archive = Archive::new(gzdecoder);
    archive.unpack(build_dir)?;

    Ok(build_dir.join("julius-4.6"))
}
