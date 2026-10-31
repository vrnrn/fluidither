use fluidither::dither::{cell_runs, dither_tones, Matrix, Run};
use fluidither::dither_image::tones_from;
use fluidither::feedback::normalized;
use fluidither::theme::cap_nudge;

#[test]
fn a_flat_tone_inks_that_share_of_the_cells() {
    let map = Matrix::Bayer8.map();
    for tone in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let tones = vec![tone; 64];
        let mut cells = vec![0u8; 64];
        dither_tones(&tones, 8, 8, map, &[0u8, 1], &mut cells);
        let inked: u32 = cells.iter().map(|cell| *cell as u32).sum();
        assert_eq!(inked as f64, tone * 64.0);
    }
}

#[test]
fn runs_merge_neighbours_and_cover_every_inked_cell() {
    #[rustfmt::skip]
    let cells = [
        1, 1, 0, 1,
        0, 0, 0, 0,
        1, 1, 1, 1,
    ];
    let mut runs = Vec::new();
    cell_runs(&cells, 4, 3, 0.0, &mut runs);
    assert_eq!(
        runs,
        vec![
            Run { x: 0, y: 0, len: 2 },
            Run { x: 3, y: 0, len: 1 },
            Run { x: 0, y: 2, len: 4 },
        ]
    );
}

#[test]
fn runs_are_cut_to_a_rounded_corner() {
    let cells = vec![1u8; 100];
    let mut runs = Vec::new();
    cell_runs(&cells, 10, 10, 4.0, &mut runs);
    let (top, middle) = (runs[0], runs[5]);
    assert!(top.x > 0 && top.x + top.len < 10);
    assert_eq!((middle.x, middle.len), (0, 10));
    assert_eq!(runs.len(), 10);
}

#[test]
fn cap_nudge_moves_capitals_to_the_middle_of_the_line() {
    // A font with a tall ascent leaves its capitals low in the line, so they move up.
    assert!((cap_nudge(1.1, 0.2, 0.7) + 0.1).abs() < 1e-6);
    // A deep descent leaves them high, so they move down.
    assert!((cap_nudge(0.8, 0.4, 0.7) - 0.15).abs() < 1e-6);
    // Balanced metrics need nothing.
    assert!(cap_nudge(0.95, 0.25, 0.7).abs() < 1e-6);
}

#[test]
fn sparkline_values_scale_to_their_own_range() {
    assert_eq!(normalized(&[10.0, 20.0, 15.0]), vec![0.0, 1.0, 0.5]);
    assert_eq!(normalized(&[5.0, 5.0]), vec![0.0, 0.0], "a flat line stays finite");
}

#[test]
fn ink_follows_the_part_of_the_picture_that_matches_it() {
    let picture = image::GrayAlphaImage::from_raw(2, 1, vec![0, 255, 255, 255]).unwrap();
    assert_eq!(tones_from(&picture, 2, 1, false), vec![1.0, 0.0]);
    assert_eq!(tones_from(&picture, 2, 1, true), vec![0.0, 1.0], "light ink follows the bright parts");
    let see_through = image::GrayAlphaImage::from_raw(1, 1, vec![0, 0]).unwrap();
    assert_eq!(tones_from(&see_through, 1, 1, false), vec![0.0], "transparency stays paper");
}

#[test]
fn the_halftone_screen_holds_every_rank_once() {
    let map = Matrix::Halftone.map();
    let mut sorted = map.thresholds.clone();
    sorted.sort_by(f64::total_cmp);
    sorted.dedup();
    assert_eq!(sorted.len(), 64);
}
