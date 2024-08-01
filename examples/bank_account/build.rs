use std::fs;

fn main() {
    let methods = risc0_build::embed_methods();
    for method in methods {
        let _ = fs::write(format!("target/{}.elf", method.name), method.elf);
    }
}
