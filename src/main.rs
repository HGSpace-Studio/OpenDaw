mod ui;
use std::io;
fn main() {
    if let Err(err) = ui::run() {
        eprintln!("都报错了你还想干什么: {err}");
        io::stdin()
            .read_line(&mut String::new())
            .expect("都报错了你还想干什么");
        std::process::exit(1);
    }
}
