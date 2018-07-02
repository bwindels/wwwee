use std::process::Command;
use std::env;

const RPI_RUST_TARGET : &'static str = "arm-unknown-linux-gnueabihf";

fn main() {
  let root = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
  let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
  let target = env::var("TARGET").expect("TARGET not set");

  let (cc, ar, ld) = match target.as_str() {
    RPI_RUST_TARGET =>
      (rpi_cmd("cc", root.as_str()), rpi_cmd("ar", root.as_str()), rpi_cmd("ld", root.as_str())),
    _ =>
      ("cc".to_owned(), "ar".to_owned(), "ld".to_owned())
  };

  
  Command::new(cc.as_str())
    .args(&["src/io/sources/file/linux/aio.c", "-c", "-fPIC", "-o"])
    .arg(&format!("{}/aio.o", out_dir))
    .status().expect("cc failed");
  Command::new(ar.as_str())
    .args(&["crus", "libwwwee-aio.a", "aio.o"])
    .current_dir(out_dir.as_str())
    .status().expect("ar failed");

  const BEAR_SSL_ROOT : &'static str = "src/tls/bearssl/lib/BearSSL";
  Command::new("make")
    .args(&["-f", "mk/SingleUnix.mk"])
    .env("WWWEE_CC", cc)
    .env("WWWEE_AR", ar)
    .env("WWWEE_LD", ld)
    .current_dir(format!("{}/{}", &root, BEAR_SSL_ROOT).as_str())
    .output().map_err(child_failed).and_then(output_to_success).expect("bearssl build failed");
  Command::new("ln")
    .args(&[
      format!("{}/build/libbearssl.a", BEAR_SSL_ROOT).as_str(),
      out_dir.as_str()])
    .current_dir(root.as_str())
    .status().expect("ln libbearssl.a failed");
  
  println!("cargo:rustc-link-search=native={}", out_dir);
  println!("cargo:rustc-link-lib=static=wwwee-aio");
  println!("cargo:rustc-link-lib=static=bearssl");

  let git_output = Command::new("git").args(&["rev-parse", "HEAD"])
    .output().expect("getting git hash failed, git meta data missing?");
  let git_hash = String::from_utf8(git_output.stdout).unwrap();
  println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}

fn rpi_cmd(cmd: &'static str, root: &str) -> String {
  const RPI_CC_TARGET : &'static str = "arm-linux-gnueabihf";
  
  format!("{root}/tools/cross_compilers/rpi/arm-bcm2708/{cc_target}/bin/{cc_target}-{cmd}",
    root = root,
    cc_target = RPI_CC_TARGET,
    cmd = cmd)
}

fn output_to_success(output: std::process::Output) -> Result<(), String> {
  if output.status.success() {
    Ok( () )
  }
  else {
    Err( String::from_utf8(output.stderr).map_err(|_| "stdout was invalid utf8".to_owned() )? )
  }
}

fn child_failed(err: std::io::Error) -> String {
  err.to_string()
}
