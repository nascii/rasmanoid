use stdweb::web::{CanvasRenderingContext2d};

const BAT_Y: f64 = 15.;         // [px]
const BAT_WIDTH: f64 = 80.;     // [px]
const BAT_HEIGHT: f64 = 20.;    // [px]
const BAT_ACCEL: f64 = 500.;    // [px/sec]
const BAT_BRAKING: f64 = 0.98;

const BALL_RADIUS: f64 = 20.;   // [px]
const BALL_BRAKING: f64 = 0.9;
const BALL_ACCEL: f64 = 1.05;

pub struct State {
    shape: (f64, f64),
    bat: Bat,
    ball: Ball,
    failed: bool,
}

struct Bat {
    x: f64,
    v: f64,
}

struct Ball {
    x: f64,
    y: f64,
    vx: f64,
    vy: f64,
}

impl State {
    pub fn new(width: u32, height: u32) -> State {
        let width = width as f64;
        let height = height as f64;

        State {
            shape: (width, height),
            bat: Bat {
                x: 0.5 * width,
                v: 0.,
            },
            ball: Ball {
                // TODO: randomize the velocity.
                x: 0.5 * width,
                y: 0.5 * height,
                vx: 200.,
                vy: 180.,
            },
            failed: false,
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
    if state.failed {
        return state;
    }

    let bat_a = if input.left { -BAT_ACCEL } else if input.right { BAT_ACCEL } else { 0. };
    let bat_v = BAT_BRAKING * (state.bat.v + bat_a * input.dt);
    let bat_x = (0.5 * BAT_WIDTH).max(state.bat.x + bat_v * input.dt).min(state.shape.0 - 0.5 * BAT_WIDTH);

    let bat = Bat {
        x: bat_x,
        v: bat_v,
    };

    let ball = collide_with_bat(state.ball, &bat);
    let failed = collide_with_danger_zone(&ball);
    let mut ball = collide_with_walls(ball, state.shape);

    ball.x += ball.vx * input.dt;
    ball.y += ball.vy * input.dt;

    State {
        shape: state.shape,
        bat,
        ball,
        failed,
    }
}

fn collide_with_walls(mut ball: Ball, shape: (f64, f64)) -> Ball {
    let (w, h) = shape;
    let r = BALL_RADIUS;

    if ball.x + r >= w {
        ball.vx = -ball.vx;
        ball.x = w - r;
    }

    if ball.x - r <= 0. {
        ball.vx = -ball.vx;
        ball.x = r;
    }

    if ball.y + r >= h {
        ball.vy = -ball.vy;
        ball.y = h - r;
    }

    ball
}

fn collide_with_danger_zone(ball: &Ball) -> bool {
    ball.y - BALL_RADIUS <= 0.
}

fn collide_with_bat(mut ball: Ball, bat: &Bat) -> Ball {
    // TODO: the corner case.

    let by = BAT_Y + 0.5 * BAT_HEIGHT;
    let bx0 = bat.x - 0.5 * BAT_WIDTH;
    let bx1 = bat.x + 0.5 * BAT_WIDTH;

    let r = 0.95 * BALL_RADIUS;

    if !(ball.y - r <= by && bx0 <= ball.x && ball.x <= bx1) {
        return ball;
    }

    ball.vy = -ball.vy;
    ball.y = by + r;

    if bat.v * ball.vx > 0. {
        ball.vx /= BALL_BRAKING;
    } else {
        ball.vx *= BALL_BRAKING;
    }

    ball.vx *= BALL_ACCEL;
    ball.vy *= BALL_ACCEL;

    ball
}

// RENDER

pub fn render(ctx: &CanvasRenderingContext2d, state: &State) {
    let (win_w, win_h) = state.shape;

    let bat_x = state.bat.x - 0.5 * BAT_WIDTH;
    let bat_y = BAT_Y - 0.5 * BAT_HEIGHT;

    let Ball {x: ball_x, y: ball_y, ..} = state.ball;

    js! { @(no_return)
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

    if state.failed {
        let text_x = 0.2 * win_w;
        let text_y = 0.5 * win_h;

        js! { @(no_return)
            @{ctx}.setTransform(1, 0, 0, 1, 0, 0);
            @{ctx}.fillText("GAME OVER", @{text_x}, @{text_y});
        }
    }
}
