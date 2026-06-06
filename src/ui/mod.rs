use vizia::prelude::*;

pub fn run() -> Result<(), String> {
    Application::new(|cx| {
        Label::new(cx, "Hello, world!");
    })
    .title("OpenDAW")
    .inner_size((800, 480))
    .run()
    .map_err(|err| err.to_string())
}
