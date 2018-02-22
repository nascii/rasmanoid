pub const BAT_Y: f64 = 15.;         // [px]
pub const BAT_WIDTH: f64 = 80.;     // [px]
pub const BAT_HEIGHT: f64 = 8.;     // [px]
pub const BAT_ACCEL: f64 = 900.;    // [px/sec]
pub const BAT_BRAKING: f64 = 0.98;
pub const BAT_MAX_SPEED: f64 = 1700.0;  // [px/sec]

pub struct Bat {
    pub x: f64,
    pub v: f64,
}

pub const BALL_RADIUS: f64 = 10.;   // [px]
pub const BALL_BRAKING: f64 = 0.5;
pub const BALL_ACCEL: f64 = 1.01;
pub const BALL_DEF_VX: f64 = 30.;
pub const BALL_DEF_VY: f64 = 300.;

pub struct Ball {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
}

pub const BLOCK_WIDTH: f64 = 40.;   // [px]
pub const BLOCK_HEIGHT: f64 = 20.;  // [px]

#[derive(Serialize)]
pub struct Block {
    pub x: f64,
    pub y: f64,
}

js_serializable!(Block);
