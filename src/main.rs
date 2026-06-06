mod ui;

fn main() {
    if let Err(err) = ui::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}
