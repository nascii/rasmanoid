use objects::Block;

pub const WIDTH: f64 = 600.;
pub const HEIGHT: f64 = 800.;

const X_CELLS_COUNT: u32 = 14;
const Y_CELLS_COUNT: u32 = 10;

const CELL_WIDTH: f64 = WIDTH / X_CELLS_COUNT as f64;
const CELL_HEIGHT: f64 = 0.5 * HEIGHT / Y_CELLS_COUNT as f64;

const MAPS: &str = include_str!("maps.txt");

pub type Map = Vec<Block>;

pub fn generate_maps() -> Vec<Map> {
    MAPS.replace("\r\n", "\n")
        .split("\n---\n")
        .map(generate_map)
        .collect()
}

fn generate_map(map: &str) -> Map {
    map.lines()
        .enumerate()
        .flat_map(|(i, line)|
            line
                .chars()
                .enumerate()
                .filter_map(move |(j, ch)| generate_block(ch, i, j))
        )
        .collect()
}

fn generate_block(ch: char, i: usize, j: usize) -> Option<Block> {
    assert!(j < 14);
    assert!(i < 14);

    if ch == ' ' {
        return None;
    }

    assert_eq!(ch, 'x');

    Some(Block {
        x: j as f64 * CELL_WIDTH + 0.5 * CELL_WIDTH,
        y: HEIGHT - i as f64 * CELL_HEIGHT - 0.5 * CELL_HEIGHT,
    })
}
