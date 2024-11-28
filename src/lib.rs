#![no_std]

extern crate alloc;

mod app;
mod esp32_s3_box;

#[esp32_s3_box::entry]
fn main() -> ! {
    esp32_s3_box::init();
    app::App::new().run().unwrap();
    panic!("The event loop should not return");
}
