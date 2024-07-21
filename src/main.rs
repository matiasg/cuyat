use cursive::theme::ColorStyle;
use cursive::views::Canvas;
use cursive::Printer;
use cursive::{theme::Color, view::Resizable};

mod draw;
use cuyat::{FoV, Sky, Star};

fn data() -> (Sky, FoV) {
    let sky = Sky::from(vec![
        Star::new(0.0, 1.0, 2.0),
        Star::new(3.0, 8.0, 5.0),
        Star::new(-1.0, 1.0, 2.0),
        Star::new(-3.0, 4.0, 5.0),
    ]);
    let fov = FoV::new(2.5, 2.5);
    (sky, fov)
}

fn draw(_: &(), p: &Printer) {
    let x_max = p.size.x as u8;
    let y_max = p.size.y as u8;

    let style = ColorStyle::new(Color::Rgb(128, 128, 128), Color::Rgb(0, 0, 255));

    let (sky, fov) = data();
    for fps in fov.project_sky_to_screen(sky, x_max, y_max).iter() {
        p.with_color(style, |printer| {
            printer.print((fps.0, fps.1), "*");
        });
    }
}

fn main() {
    let mut siv = cursive::default();
    siv.add_global_callback('q', |s| s.quit());
    siv.add_layer(Canvas::new(()).with_draw(draw).fixed_size((20, 10)));
    siv.run();
}
