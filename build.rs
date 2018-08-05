
use std::process::Command;
use std::env;
use std::path::Path;

fn main() {
    let out_dir = "./target/x86_64-unknown-efi/debug";
    
    let srcs=["x86_64.S"];
    for i in 0..srcs.len(){
        let src_stem=Path::new(srcs[i]).file_stem().unwrap().to_str().unwrap();
        Command::new("gcc").args(&[&format!("src/ffi/{}",srcs[i]),"-c", "-fPIC", "-o"])
                        .arg(&format!("{}/{}.o", out_dir,src_stem))
                        .status().unwrap();
    }
}