// build.rs

use std::path::Path;
use std::process::Command;
use std::{env, fs, io::Write};

fn main() {
    let mut path = env::current_dir().unwrap();
    path.push("codegen.py");
    let codegen_bin = path.to_str().unwrap();

    // for each templates, perform codegen
    let template_dir = fs::read_dir("templates/").unwrap();
    let mut modules = Vec::new();
    for template in template_dir {
        let status = Command::new(codegen_bin)
            .args([
                template.as_ref().unwrap().path().to_str().unwrap(),
                "--dir",
                "src",
            ])
            .current_dir(env::current_dir().unwrap())
            .status()
            .expect("Failed to execute codegen");
        if !status.success() {
            panic!("Failed to generate decoder for {:?}", template);
        }
        let basename = template
            .unwrap()
            .path()
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .split('.')
            .next()
            .unwrap()
            .to_string() + "_decoder;";
        modules.push(format!("pub mod {}", basename));
    }
    let mod_dest = Path::new("src").join("mod.rs");
    let mut f = fs::File::create(&mod_dest).expect("Could not create mod.rs");
    write!(&mut f, "{}", modules.join("\n")).expect("Could not write to file");
}
