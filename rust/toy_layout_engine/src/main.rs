extern crate image;

use std::default::Default;
use std::fs::File;
use std::io::Read;
use std::path::Path;

mod css;
mod dom;
mod html;
mod layout;
mod painting;
mod style;

fn main() {
    // Read input files:
    let read_source = |filename: &str| {
        let mut buf = String::new();
        File::open(&Path::new(filename))
            .unwrap()
            .read_to_string(&mut buf)
            .unwrap();
        buf
    };
    let html = read_source("examples/test.html");
    let css = read_source("examples/test.css");

    let mut init_view: layout::Dimensions = Default::default();
    init_view.content.width = 800.0;
    init_view.content.height = 600.0;

    // Parsing and rendering:
    let root_node = html::parse(html);
    let stylesheet = css::parse(css);
    let style_root = style::style_tree(&root_node, &stylesheet);
    let layout_root = layout::layout_tree(&style_root, init_view);

    // Create the output file:
    let filename = "output.png".to_string();

    // Save an image:
    // Since we don't have an actual window, hard-code the "viewport" size.
    let mut viewport: layout::Dimensions = Default::default();
    viewport.content.width = 800.0;
    viewport.content.height = 600.0;
    let canvas = painting::paint(&layout_root, viewport.content);
    let (w, h) = (canvas.width as u32, canvas.height as u32);
    let buffer: Vec<image::Rgba<u8>> = unsafe { std::mem::transmute(canvas.pixels) };
    let img = image::ImageBuffer::from_fn(
        w,
        h,
        Box::new(|x: u32, y: u32| buffer[(y * w + x) as usize]),
    );

    let result = img.save(&Path::new(&filename));
    match result {
        Ok(_) => println!("Saved output as {}", filename),
        Err(e) => println!("Error saving output as {}, {}", filename, e),
    }

    // Debug output:
    // println!("{}", layout_root.dimensions);
    // println!("{}", display_list);
}
