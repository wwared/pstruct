use std::{env, io, path, process};

fn main() -> io::Result<()> {
    println!("cargo:rerun-if-changed=src/test.zs");
    println!("cargo:rerun-if-changed=src/test.rs");

    let root = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let path = path::Path::new(&root).join("src/test.zs");

    // this executes the binary directly, assuming you installed it
    process::Command::new("pst")
        .arg("rust")
        .arg(path.into_os_string().into_string().unwrap())
        .status()?;

    // use std::fs;
    // use io::Write;
    // let mut out_file = path.to_owned();
    // out_file.set_extension("rs");
    // let file_contents = fs::read_to_string(&path)?;

    // let file = pstruct::parser::parse_file(file_contents.as_str()).unwrap();
    // let output = pstruct_rust::render_file(&file);

    // // NOTE: the output looks ungodly awful without passing it through rustfmt, which is not available as a library so you need to shell out
    // let mut child = process::Command::new("rustfmt")
    //     .stdin(process::Stdio::piped())
    //     .stdout(process::Stdio::piped())
    //     .stderr(process::Stdio::piped())
    //     .spawn()?;

    // let mut stdin = child.stdin.take().unwrap();
    // write!(stdin, "{output}")?;
    // stdin.flush()?;
    // drop(stdin);

    // let process::Output {
    //     status,
    //     stdout,
    //     stderr: _,
    // } = child.wait_with_output()?;
    // let stdout = String::from_utf8_lossy(&stdout);

    // assert!(status.success());
    // let output: String = stdout.into();

    // fs::write(out_file.as_path(), &output)?;

    Ok(())
}
