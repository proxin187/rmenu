
fn main() {
    println!("cargo:rustc-link-lib=X11");
    println!("cargo:rustc-link-lib=Xinerama");
    println!("cargo:rustc-link-lib=Xft");
}

