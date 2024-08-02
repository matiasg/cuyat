use cuyat::view::SkyView;

fn main() {
    // for random sky use this
    // let (sky_view, score_rc) = SkyView::new(None, 200);
    let (sky_view, score_rc) = SkyView::new(Some(String::from("bsc5.csv")), 400);
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
        score.total.iter().sum::<f32>(),
        score.total.len(),
        score.get_score()
    );
}
