//! Embeds the icon and version info into Windows executables, so Explorer, the
//! taskbar and the Start menu show Void Armada's icon instead of a blank one.

fn main() {
    println!("cargo:rerun-if-changed=assets/icon/VoidArmada.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/icon/VoidArmada.ico").set("ProductName", "Void Armada").set("FileDescription", "Void Armada");
        res.compile().expect("embedding the Windows icon");
    }
}
