use std::collections::HashMap;
use std::io;
use std::path::Path;

use insights_runtime_extractor::config;
use insights_runtime_extractor::file;

fn main() -> io::Result<()> {
    let exec = std::env::args()
        .nth(1)
        .expect("Missing argument for executable");
    let name = std::env::args().nth(2).expect("Missing argument for name");
    let out = std::env::args()
        .nth(3)
        .expect("Missing argument for output directory");

    let _config = config::get_config(&out);

    println!("🔎 Fingerprinting {} Process to {}", exec, out);

    match file::get_executable_version_output(&exec) {
        Ok(version) => {
            let mut entries = HashMap::new();
            entries.insert(String::from("runtime-kind"), String::from(name));
            entries.insert(String::from("runtime-kind-version"), version);

            file::write_entries(Path::new(&out), "runtime-kind-fingerprints.txt", &entries)
        }
        Err(err) => Err(err),
    }
}
