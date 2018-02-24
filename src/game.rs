use stdweb::web::{CanvasRenderingContext2d};

use objects::*;
use maps::{Map, WIDTH, HEIGHT};

pub struct State {
    map: Map,
    bat: Bat,
    ball: Ball,
    status: Status,
}

enum Status {
    NotStarted,
    Started,
    Failed,
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
                y: BAT_Y + 0.5 * BAT_HEIGHT + BALL_RADIUS,
                vx: 0.,
                vy: 0.,
            },
            status: Status::NotStarted,
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
    match state.status {
        Status::NotStarted => waiting(state, input),
        Status::Started => flying(state, input),
        Status::Failed => state,
    }
}

fn waiting(mut state: State, input: Input) -> State {
    if input.left || input.right {
        state.ball.vx = if input.left { -BALL_DEF_VX } else { BALL_DEF_VX };
        state.ball.vy = BALL_DEF_VY;

        state.status = Status::Started;
    }

    state
}

fn flying(state: State, input: Input) -> State {
    let bat_a = if input.left { -BAT_ACCEL } else if input.right { BAT_ACCEL } else { 0. };
    let mut bat_v = (-BAT_MAX_SPEED).max(BAT_BRAKING * (state.bat.v + bat_a * input.dt)).min(BAT_MAX_SPEED);

    let mut bat_x = state.bat.x + bat_v * input.dt;
    let min_bat_x = 0.5 * BAT_WIDTH;
    let max_bat_x = WIDTH - 0.5 * BAT_WIDTH;

    if bat_x < min_bat_x {
        bat_x = min_bat_x;
        bat_v = 0.0;
    }

    if bat_x > max_bat_x {
        bat_x = max_bat_x;
        bat_v = 0.0;
    }

    let bat = Bat {
        x: bat_x,
        v: bat_v,
    };

    let ball = collide_with_bat(state.ball, &bat);
    let failed = collide_with_danger_zone(&ball);
    let ball = collide_with_walls(ball);

    let (mut ball, map) = if ball.y > 0.5 * HEIGHT {
        collide_with_blocks(ball, state.map)
    } else {
        (ball, state.map)
    };

    ball.x += ball.vx * input.dt;
    ball.y += ball.vy * input.dt;

    State {
        map,
        bat,
        ball,
        status: if failed { Status::Failed } else { state.status },
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

    ball.vx += bat.v * BALL_BRAKING;

    ball.vx *= BALL_ACCEL;
    ball.vy *= BALL_ACCEL;

    ball
}

fn collide_with_blocks(mut ball: Ball, mut map: Map) -> (Ball, Map) {
    let mut remove_idx = None;

    for (idx, block) in map.iter().enumerate() {
        let (new_ball, detected) = collide_with_block(ball, &block);

        ball = new_ball;

        if detected {
            remove_idx = Some(idx);
            break;
        }
    }

    if let Some(idx) = remove_idx {
        map.swap_remove(idx);
    }

    (ball, map)
}

fn collide_with_block(mut ball: Ball, block: &Block) -> (Ball, bool) {

    /*   C₁ |   A₁  |  C₂
     * -----#########-----
     *   B₁ #########  B₂
     * -----#########-----
     *   C₄ |   A₂  |  C₃
     */

    let left = block.x - 0.5 * BLOCK_WIDTH;
    let right = block.x + 0.5 * BLOCK_WIDTH;
    let bottom = block.y - 0.5 * BLOCK_HEIGHT;
    let top = block.y + 0.5 * BLOCK_HEIGHT;

    let mut detected = true;

    if ball.x >=left && ball.x <= right { // A
        let ball_top = ball.y + BALL_RADIUS;
        let ball_bottom = ball.y - BALL_RADIUS;

        if ball_top >= bottom && ball_bottom <= top {
            /* Наша реакция на столкновение зависит от направления подлёта.
             * Выталкивать шар нужно в сторону подлёта, то есть против направления скорости
             * (вообще нужно смотреть на предыдущее положение, но мы ограничимся скоростью)
             */

            // скорость положительная -> шар подлетал снизу вверх -> выталкиваем вниз.
            if ball.vy > 0. {
                ball.y = ball_bottom;
            } else {
                ball.y = ball_top;
            }
            ball.vy = -ball.vy;
        } else {
            detected = false;
        }
    } else if ball.y >= bottom && ball.y <= top { // B
        let ball_right = ball.x + BALL_RADIUS;
        let ball_left = ball.x - BALL_RADIUS;

        if ball_right >= left && ball_left <= right {
            ball.x = if ball.vx > 0. { ball_left } else { ball_right };
            ball.vx = -ball.vx;
        } else {
            detected = false;
        }
    } else { // C
        let bx = ball.x - block.x;
        let by = ball.y - block.y;

        // (right, top) corner
        let corner_x = 0.5 * BLOCK_WIDTH;
        let corner_y = 0.5 * BLOCK_HEIGHT;

        let dx = corner_x - bx.abs();
        let dy = corner_y - by.abs();

        let dist2_corner_ball_center = dx * dx + dy * dy;

        if dist2_corner_ball_center < BALL_RADIUS * BALL_RADIUS {
            let norm_x = (right - ball.x).abs() * bx.signum();
            let norm_y = (top - ball.y).abs() * by.signum();

            let (vx, vy) = reflect((ball.vx, ball.vy), (norm_x, norm_y));

            ball.vx = vx;
            ball.vy = vy;
        } else {
            detected = false;
        }
    }

    (ball, detected)
}

// pos - norm * 2(pos, norm)
fn reflect(pos: (f64, f64), norm: (f64, f64)) -> (f64, f64) {
    let dot = pos.0 * norm.0 + pos.1 * norm.1;
    let norm_l2 = norm.0 * norm.0 + norm.1 * norm.1;

    (
        pos.0 - 2. * dot * norm.0 / norm_l2,
        pos.1 - 2. * dot * norm.1 / norm_l2
    )
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

    if let Status::Failed = state.status {
        let text_x = 0.2 * WIDTH;
        let text_y = 0.5 * HEIGHT;

        js! { @(no_return)
            @{ctx}.setTransform(1, 0, 0, 1, 0, 0);
            @{ctx}.fillText("GAME OVER", @{text_x}, @{text_y});
        }
    }
}
