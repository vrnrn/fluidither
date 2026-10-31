//! Holds the Rust engines to the numbers the JS engines produced.

use fluidither::charts::{chart_cells, ChartKind};
use fluidither::dates::{month_grid, Date};
use fluidither::text_effects::scrambled;
use fluidither::world::{fill_globe, fill_map};
use fluidither::fields::{fill_field, wave, Field};
use fluidither::dither::{dither_tones, fill_gradient, fill_sparkline, grain, Gradient, Matrix, Shape};
use fluidither::generated::{SpringConfig, GLIDE, QUICK, SETTLE};
use fluidither::spring::{step_spring, Spring};
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let path = format!("{}/../fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn matrix(name: &str) -> Matrix {
    match name {
        "bayer2" => Matrix::Bayer2,
        "bayer4" => Matrix::Bayer4,
        "bayer8" => Matrix::Bayer8,
        "blue-noise" => Matrix::BlueNoise,
        other => panic!("unknown matrix {other}"),
    }
}

fn gradient(value: &Value) -> Gradient {
    let number = |key: &str| value[key].as_f64().unwrap();
    let center = (number("centerX"), number("centerY"));
    Gradient {
        shape: match value["mode"].as_u64().unwrap() {
            0 => Shape::Linear { angle: number("angle") },
            1 => Shape::Radial { center, radius: number("radius") },
            _ => Shape::Conic { center, angle: number("angle") },
        },
        from: number("from"),
        to: number("to"),
        warp: number("warp"),
    }
}

fn hash_cells(cells: &[u32]) -> u64 {
    let mut hash: u32 = 0x811c9dc5;
    for cell in cells {
        hash = (hash ^ cell).wrapping_mul(0x01000193);
    }
    hash as u64
}

#[test]
fn dither_cells_match_javascript_exactly() {
    let fixture = fixture("dither.json");
    let cases = fixture["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let width = case["width"].as_u64().unwrap() as usize;
        let height = case["height"].as_u64().unwrap() as usize;
        let colors = case["colors"].as_u64().unwrap() as u32;
        let palette: Vec<u32> = (0..colors).collect();
        let mut tones = vec![0.0; width * height];
        let mut cells = vec![0u32; width * height];
        fill_gradient(
            &mut tones,
            width,
            height,
            &gradient(&case["gradient"]),
            case["t"].as_f64().unwrap(),
        );
        dither_tones(
            &tones,
            width,
            height,
            matrix(case["matrix"].as_str().unwrap()).map(),
            &palette,
            &mut cells,
        );
        assert_eq!(hash_cells(&cells), case["hash"].as_u64().unwrap(), "{name}");
    }
}

#[test]
fn sparkline_cells_match_javascript_exactly() {
    let fixture = fixture("dither.json");
    let case = &fixture["sparkline"];
    let width = case["width"].as_u64().unwrap() as usize;
    let height = case["height"].as_u64().unwrap() as usize;
    let values: Vec<f64> = case["values"].as_array().unwrap().iter().map(|value| value.as_f64().unwrap()).collect();
    let mut tones = vec![0.0; width * height];
    let mut cells = vec![0u32; width * height];
    fill_sparkline(&mut tones, width, height, &values, case["fade"].as_f64().unwrap());
    dither_tones(&tones, width, height, matrix(case["matrix"].as_str().unwrap()).map(), &[0, 1], &mut cells);
    assert_eq!(hash_cells(&cells), case["hash"].as_u64().unwrap());
}

#[test]
fn noise_matches_javascript_to_the_bit() {
    let fixture = fixture("dither.json");
    for sample in fixture["noise"].as_array().unwrap() {
        let x = sample["x"].as_f64().unwrap();
        let y = sample["y"].as_f64().unwrap();
        assert_eq!(grain(x, y), sample["value"].as_f64().unwrap(), "grain({x}, {y})");
    }
}

fn config(name: &str) -> SpringConfig {
    match name {
        "quick" => QUICK,
        "glide" => GLIDE,
        "settle" => SETTLE,
        other => panic!("unknown spring {other}"),
    }
}

#[test]
fn springs_trace_the_same_curve_as_javascript() {
    let fixture = fixture("spring.json");
    for entry in fixture["springs"].as_array().unwrap() {
        let name = entry["name"].as_str().unwrap();
        let config = config(name);
        let (mut position, mut velocity) = (0.0, 0.0);
        for (frame, point) in entry["trace"].as_array().unwrap().iter().enumerate() {
            let target = if frame < 6 { 100.0 } else { -40.0 };
            (position, velocity) = step_spring(&config, position, velocity, target, 1.0 / 120.0);
            let expected = (point[0].as_f64().unwrap(), point[1].as_f64().unwrap());
            assert!((position - expected.0).abs() < 1e-9, "{name} frame {frame} position");
            assert!((velocity - expected.1).abs() < 1e-7, "{name} frame {frame} velocity");
        }
        let mut whole = Spring::new(config, 0.0);
        whole.target = 1.0;
        let mut frames = 0;
        while !whole.step(1.0 / 60.0) && frames < 600 {
            frames += 1;
        }
        assert_eq!(frames, entry["framesToSettle"].as_u64().unwrap(), "{name} settle");
    }
}

#[test]
fn critically_damped_springs_never_overshoot() {
    for config in [QUICK, GLIDE] {
        let mut spring = Spring::new(config, 0.0);
        spring.target = 1.0;
        for _ in 0..240 {
            spring.step(1.0 / 240.0);
            assert!(spring.position <= 1.0);
        }
        assert!(spring.landed());
    }
}

fn field(name: &str) -> Field {
    match name {
        "noise" => Field::Noise,
        "waves" => Field::Waves,
        "plasma" => Field::Plasma,
        "ripples" => Field::Ripples,
        "tunnel" => Field::Tunnel,
        "metaballs" => Field::Metaballs,
        "orb" => Field::Orb,
        "stars" => Field::Stars,
        "aurora" => Field::Aurora,
        "contours" => Field::Contours,
        "beams" => Field::Beams,
        "eclipse" => Field::Eclipse,
        "horizon" => Field::Horizon,
        "mesh" => Field::Mesh,
        "grid" => Field::Grid,
        "ridges" => Field::Ridges,
        "silk" => Field::Silk,
        "smoke" => Field::Smoke,
        other => panic!("unknown field {other}"),
    }
}

#[test]
fn every_field_matches_javascript_exactly() {
    let fixture = fixture("fields.json");
    let cases = fixture["fields"].as_array().unwrap();
    assert!(!cases.is_empty());
    for case in cases {
        let name = case["kind"].as_str().unwrap();
        let (width, height) = (case["width"].as_u64().unwrap() as usize, case["height"].as_u64().unwrap() as usize);
        let t = case["t"].as_f64().unwrap();
        let mut tones = vec![0.0; width * height];
        let mut cells = vec![0u32; width * height];
        fill_field(&mut tones, width, height, field(name), t);
        dither_tones(&tones, width, height, Matrix::Bayer4.map(), &[0, 1], &mut cells);
        assert_eq!(hash_cells(&cells), case["hash"].as_u64().unwrap(), "{name} at {t}");
    }
    for sample in fixture["waves"].as_array().unwrap() {
        assert_eq!(wave(sample["t"].as_f64().unwrap()), sample["value"].as_f64().unwrap());
    }
}

#[test]
fn every_chart_matches_javascript_exactly() {
    let fixture = fixture("fields.json");
    for case in fixture["charts"].as_array().unwrap() {
        let name = case["kind"].as_str().unwrap();
        let kind = match name {
            "bar" => ChartKind::Bar,
            "area" => ChartKind::Area,
            "line" => ChartKind::Line,
            "donut" => ChartKind::Donut,
            "radar" => ChartKind::Radar,
            "heatmap" => ChartKind::Heatmap,
            "gauge" => ChartKind::Gauge,
            other => panic!("unknown chart {other}"),
        };
        let series: Vec<Vec<f64>> = case["series"]
            .as_array()
            .unwrap()
            .iter()
            .map(|list| list.as_array().unwrap().iter().map(|value| value.as_f64().unwrap()).collect())
            .collect();
        let (width, height) = (case["width"].as_u64().unwrap() as usize, case["height"].as_u64().unwrap() as usize);
        let mut cells = vec![0u8; width * height];
        chart_cells(kind, &series, width, height, Matrix::Bayer4.map(), &mut cells);
        let words: Vec<u32> = cells.iter().map(|cell| u32::from(*cell)).collect();
        assert_eq!(hash_cells(&words), case["hash"].as_u64().unwrap(), "{name}");
        assert!(case["inked"].as_u64().unwrap() > 0, "{name} draws something");
    }
}

#[test]
fn scrambles_match_javascript_exactly() {
    let fixture = fixture("fields.json");
    for case in fixture["scramble"].as_array().unwrap() {
        let shown = scrambled("Hello there, world", case["progress"].as_f64().unwrap(), case["seed"].as_u64().unwrap() as u32);
        assert_eq!(shown, case["text"].as_str().unwrap());
    }
}

fn parse(text: &str) -> Date {
    let parts: Vec<i64> = text.split('-').map(|part| part.parse().unwrap()).collect();
    Date::new(parts[0] as i32, parts[1] as u32, parts[2] as u32)
}

#[test]
fn dates_match_javascript_exactly() {
    let fixture = fixture("dates.json");
    for case in fixture["days"].as_array().unwrap() {
        let date = parse(case["date"].as_str().unwrap());
        assert_eq!(date.to_days(), case["days"].as_i64().unwrap());
        assert_eq!(i64::from(date.weekday()), case["weekday"].as_i64().unwrap());
        assert_eq!(Date::from_days(date.to_days()), date);
    }
    for case in fixture["months"].as_array().unwrap() {
        let grid: Vec<String> = month_grid(case["year"].as_i64().unwrap() as i32, case["month"].as_u64().unwrap() as u32).iter().map(|date| date.iso()).collect();
        let expected: Vec<&str> = case["grid"].as_array().unwrap().iter().map(|day| day.as_str().unwrap()).collect();
        assert_eq!(grid, expected);
    }
    for case in fixture["shifts"].as_array().unwrap() {
        let moved = parse(case["from"].as_str().unwrap()).plus_months(case["count"].as_i64().unwrap() as i32);
        assert_eq!(moved.iso(), case["to"].as_str().unwrap());
    }
}

#[test]
fn the_map_and_globe_match_javascript_exactly() {
    let fixture = fixture("fields.json");
    for case in fixture["world"].as_array().unwrap() {
        let name = case["kind"].as_str().unwrap();
        let turn = case["turn"].as_f64().unwrap();
        let (width, height) = (case["width"].as_u64().unwrap() as usize, case["height"].as_u64().unwrap() as usize);
        let mut tones = vec![0.0; width * height];
        let mut cells = vec![0u32; width * height];
        if name == "map" {
            fill_map(&mut tones, width, height, turn);
        } else {
            fill_globe(&mut tones, width, height, turn);
        }
        dither_tones(&tones, width, height, Matrix::Bayer4.map(), &[0, 1], &mut cells);
        assert_eq!(hash_cells(&cells), case["hash"].as_u64().unwrap(), "{name} at {turn}");
    }
}
