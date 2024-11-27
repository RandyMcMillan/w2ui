use std::{
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::Context;

#[derive(Copy, Clone)]
pub struct Paths<'a> {
    sass_in_dir: &'a Path,
    sass_out_dir: &'a Path,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occurred within the rgit build script:\n\n{:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
    let sass_in_dir = manifest_dir.join("src/sass");

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").context("OUT_DIR not set by rustc")?);
    let sass_out_dir = out_dir.join("dist");
    let sass_out_dir_style_css = sass_out_dir.join("style.css");
    println!("{}", sass_out_dir_style_css.display());

    let paths = Paths {
        sass_in_dir: &sass_in_dir,
        sass_out_dir: &sass_out_dir,
    };

    if let Ok(output) = Command::new("cp").args([sass_out_dir_style_css, "./dist".into()]).output(){};
    if let Ok(output) = Command::new("git").args(["rev-parse", "HEAD"]).output() {
        if let Ok(git_hash) = String::from_utf8(output.stdout) {
            build_scss(paths).context("Failed to build CSS stylesheets")?;
            println!("cargo:rerun-if-changed={git_hash}");
            println!("cargo:rustc-env=GIT_HASH={git_hash}");
        }
    }

    Ok(())
}

fn build_scss(paths: Paths) -> anyhow::Result<()> {
    std::fs::create_dir_all(&paths.sass_out_dir).context("Failed to create output directory")?;

    println!("cargo:rerun-if-changed={}", paths.sass_in_dir.display());

    let input_file = paths.sass_in_dir.join("style.scss");
    let output_file = paths.sass_out_dir.join("style.css");
    let format = rsass::output::Format {
        style: rsass::output::Style::Compressed,
        ..rsass::output::Format::default()
    };

    let output_content =
        rsass::compile_scss_path(&input_file, format).context("Failed to compile SASS")?;

    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)
        .context("Failed to open output file")?;
    output_file
        .write_all(&output_content)
        .context("Failed to write compiled CSS to output")?;

    Ok(())
}
