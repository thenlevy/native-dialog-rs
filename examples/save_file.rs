use native_dialog::*;

fn main() {
    let dialog = SaveFile {
        dir: None,
        name: "test",
        types: &[FileType::new("jpeg image", &["jpeg", "jpg"])],
    };
    let result = dialog.show();

    println!("{:?}", result);
}
