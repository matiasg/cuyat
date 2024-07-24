use cuyat::SkyView;

fn main() {
    let (sky_view, total) = SkyView::new(12);
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    println!(
        "\n\n\n ====>>>> total: {:?} <<<====\n\n\n",
        (*total).borrow().get_score()
    );
}
