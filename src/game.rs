use stdweb::web::{CanvasRenderingContext2d};

use objects::*;
use maps::{Map, WIDTH, HEIGHT};

pub struct State {
    map: Map,
    bat: Bat,
    ball: Ball,
    failed: bool,
}

impl State {
    pub fn new(map: Map) -> State {
        State {
            map,
            bat: Bat {
                x: 0.5 * WIDTH,
                v: 0.,
            },
            ball: Ball {
                // TODO: randomize the velocity.
                x: 0.5 * WIDTH,
                y: 0.5 * HEIGHT,
                vx: 300.,
                vy: 230.,
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
    let bat_x = (0.5 * BAT_WIDTH).max(state.bat.x + bat_v * input.dt).min(WIDTH - 0.5 * BAT_WIDTH);

    let bat = Bat {
        x: bat_x,
        v: bat_v,
    };

    let ball = collide_with_bat(state.ball, &bat);
    let failed = collide_with_danger_zone(&ball);
    let mut ball = collide_with_walls(ball);

    ball.x += ball.vx * input.dt;
    ball.y += ball.vy * input.dt;

    State {
        map: state.map,
        bat,
        ball,
        failed,
    }
}

fn collide_with_walls(mut ball: Ball) -> Ball {
    if ball.x + BALL_RADIUS >= WIDTH {
        ball.vx = -ball.vx;
        ball.x = WIDTH - BALL_RADIUS;
    }

    if ball.x - BALL_RADIUS <= 0. {
        ball.vx = -ball.vx;
        ball.x = BALL_RADIUS;
    }

    if ball.y + BALL_RADIUS >= HEIGHT {
        ball.vy = -ball.vy;
        ball.y = HEIGHT - BALL_RADIUS;
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
    let bat_x = state.bat.x - 0.5 * BAT_WIDTH;
    let bat_y = BAT_Y - 0.5 * BAT_HEIGHT;

    let Ball {x: ball_x, y: ball_y, ..} = state.ball;

    js! { @(no_return)
        var c = @{ctx};

        // Normalize the coordinate system.
        c.setTransform(1, 0, 0, -1, 0, @{HEIGHT});

        // Clear the canvas.
        c.clearRect(0, 0, @{WIDTH}, @{HEIGHT});

        // Draw the bat.
        c.fillRect(@{bat_x}, @{bat_y}, @{BAT_WIDTH}, @{BAT_HEIGHT});

        // Draw the ball.
        c.beginPath();
        c.arc(@{ball_x}, @{ball_y}, @{BALL_RADIUS}, 0, 2 * Math.PI, false);
        c.lineWidth = 2;
        c.stroke();

        // Draw blocks.
        var blocks = @{&state.map};
        var bw = @{BLOCK_WIDTH};
        var bh = @{BLOCK_HEIGHT};

        for (var i = 0; i < blocks.length; ++i)
            c.fillRect(blocks[i].x - 0.5 * bw, blocks[i].y - 0.5 * bh, bw, bh);
    }

    if state.failed {
        let text_x = 0.2 * WIDTH;
        let text_y = 0.5 * HEIGHT;

        js! { @(no_return)
            @{ctx}.setTransform(1, 0, 0, 1, 0, 0);
            @{ctx}.fillText("GAME OVER", @{text_x}, @{text_y});
        }
    }
}
