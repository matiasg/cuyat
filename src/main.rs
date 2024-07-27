use cuyat::sky::Sky;
use cuyat::view::SkyView;

fn main() {
    let sky = Sky::from_converted_file("bsc5.csv").with_random_quaternion();
    let (sky_view, score_rc) = SkyView::new_from(sky, false);
    // let (sky_view, score_rc) = SkyView::new(24);
    let mut siv = cursive::default();
    siv.add_layer(sky_view);
    siv.add_global_callback('q', |s| s.quit());
    siv.run();
    let score = (*score_rc).borrow();
    println!(
        "


        ========
        moves: {}
        total: {:.6}
        games: {}
        --------
        score: {:.6}
        ========

        ",
        score.counted_moves,
        score.total,
        score.games,
        score.get_score()
    );
}
