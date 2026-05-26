// This is a meta-package that installs all AIMF CLI tools
// Usage: cargo install aimf-bundle
// This installs: aimf, aaud, aimg, avid

fn main() {
    println!("AIMF Bundle installed!");
    println!("Available commands:");
    println!("  aimf  - Universal tool (handles all media types)");
    println!("  aaud  - Audio-only tool");
    println!("  aimg  - Image-only tool");
    println!("  avid  - Video-only tool");
    println!();
    println!("Try: aimf --help");
}
