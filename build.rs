fn main() {
    slint_build::compile("ui/window.slint").unwrap();
    if cfg!(target_os = "windows") {
        let mut res = winres::WindowsResource::new();
        res.set_icon("ui/2.ico"); // Replace this with the filename of your .ico file.
        res.compile().unwrap();
      }
}