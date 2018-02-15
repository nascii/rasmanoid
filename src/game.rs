use stdweb::web::{CanvasRenderingContext2d};

const BAT_Y: f64 = 15.;         // [px]
const BAT_WIDTH: f64 = 80.;     // [px]
const BAT_HEIGHT: f64 = 20.;    // [px]
const BAT_ACCEL: f64 = 500.;    // [px/sec]
const BAT_BRAKING: f64 = 0.98;

const BALL_RADIUS: f64 = 20.;   // [px]

pub struct State {
    shape: (f64, f64),
    bat_x: f64,
    bat_v: f64,
}

impl State {
    pub fn new(width: u32, height: u32) -> State {
        State {
            shape: (width as f64, height as f64),
            bat_x: 300.,
            bat_v: 0.,
        }
    }
}

pub struct Input {
    pub dt: f64,
    pub left: bool,
    pub right: bool,
}

// SIMULATE

pub fn simulate(state: State, input: Input) -> State {
    let bat_a = if input.left { -BAT_ACCEL } else if input.right { BAT_ACCEL } else { 0. };
    let bat_v = BAT_BRAKING * (state.bat_v + bat_a * input.dt);
    let bat_x = (0.5 * BAT_WIDTH).max(state.bat_x + bat_v * input.dt).min(state.shape.0 - 0.5 * BAT_WIDTH);

    State {
        shape: state.shape,
        bat_x,
        bat_v,
    }
}

// RENDER

pub fn render(ctx: &CanvasRenderingContext2d, state: &State) {
    let (win_w, win_h) = state.shape;

    let bat_x = state.bat_x - 0.5 * BAT_WIDTH;
    let bat_y = BAT_Y - 0.5 * BAT_HEIGHT;

    let ball_x = 300;
    let ball_y = 300;

    js! {
        var c = @{ctx};

        // Normalize the coordinate system.
        c.setTransform(1, 0, 0, -1, 0, @{win_h});

        // Clear the canvas.
        c.clearRect(0, 0, @{win_w}, @{win_h});

        // Draw the bat.
        c.fillRect(@{bat_x}, @{bat_y}, @{BAT_WIDTH}, @{BAT_HEIGHT});

        // Draw the ball.
        c.beginPath();
        c.arc(@{ball_x}, @{ball_y}, @{BALL_RADIUS}, 0, 2 * Math.PI, false);
        c.lineWidth = 2;
        c.stroke();
    }
}
