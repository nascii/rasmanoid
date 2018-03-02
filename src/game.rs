use std::iter;
use std::f64::consts::PI;

use stdweb::{Reference, UnsafeTypedArray};
use stdweb::web::{document, CanvasRenderingContext2d};
use stdweb::web::html_element::CanvasElement;
use stdweb::unstable::TryInto;

use objects::*;
use maps::{Map, WIDTH, HEIGHT};

pub struct State {
    map: Map,
    bat: Bat,
    ball: Ball,
    status: Status,
    buffer: Vec<f32>,   // [bat, ball, ...blocks]
}

enum Status {
    NotStarted,
    Started,
    Failed,
}

impl State {
    pub fn new(map: Map) -> State {
        let block_count = map.len();
        let mut buffer = vec![0.; 2 * 6 * (block_count + 2)];

        let bat = Bat {
            x: 0.5 * WIDTH,
            v: 0.,
        };

        let ball = Ball {
            // TODO: randomize the velocity.
            x: 0.5 * WIDTH,
            y: BAT_Y + 0.5 * BAT_HEIGHT + BALL_RADIUS,
            vx: 0.,
            vy: 0.,
        };

        fill_buffer(&mut buffer, &bat, &ball, &map);

        State {
            map,
            bat,
            ball,
            status: Status::NotStarted,
            buffer,
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

    let mut buffer = state.buffer;

    fill_buffer(&mut buffer, &bat, &ball, &map);

    State {
        map,
        bat,
        ball,
        status: if failed { Status::Failed } else { state.status },
        buffer,
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

            // TODO: искать настоящую точку пересечения прямых вектора скорости и top/bottom,
            // а не выталкивать по перпендикуляру.
            ball.y = if ball.vy > 0. { bottom - BALL_RADIUS } else { top + BALL_RADIUS };
            ball.vy = -ball.vy;
        } else {
            detected = false;
        }
    } else if ball.y >= bottom && ball.y <= top { // B
        let ball_right = ball.x + BALL_RADIUS;
        let ball_left = ball.x - BALL_RADIUS;

        if ball_right >= left && ball_left <= right {
            ball.x = if ball.vx > 0. { left - BALL_RADIUS } else { right + BALL_RADIUS };
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

fn fill_buffer(buffer: &mut [f32], bat: &Bat, ball: &Ball, map: &Map) {
    let mut it = buffer.chunks_mut(12);

    fill_buffer_with_rect(it.next().unwrap(), bat.x, BAT_Y, 0.5 * BAT_WIDTH, 0.5 * BAT_HEIGHT);
    fill_buffer_with_rect(it.next().unwrap(), ball.x, ball.y, BALL_RADIUS, BALL_RADIUS);

    for (block, buf) in map.iter().zip(it) {
        fill_buffer_with_rect(buf, block.x, block.y, 0.5 * BLOCK_WIDTH, 0.5 * BLOCK_HEIGHT);
    }
}

fn fill_buffer_with_rect(buffer: &mut [f32], x: f64, y: f64, hw: f64, hh: f64) {
    assert_eq!(buffer.len(), 12);

    buffer[0] = (x - hw) as f32;
    buffer[1] = (y + hh) as f32;
    buffer[2] = (x - hw) as f32;
    buffer[3] = (y - hh) as f32;
    buffer[4] = (x + hw) as f32;
    buffer[5] = (y + hh) as f32;
    buffer[6] = (x - hw) as f32;
    buffer[7] = (y - hh) as f32;
    buffer[8] = (x + hw) as f32;
    buffer[9] = (y + hh) as f32;
    buffer[10] = (x + hw) as f32;
    buffer[11] = (y - hh) as f32;
}

// PREPARE

const TEX_COORDS: &[f32] = &[0., 0., 0., 1., 1., 0., 0., 1., 1., 0., 1., 1.];

const PLACE_VERT: &str = include_str!("place.vert");
const DRAW_FRAG: &str = include_str!("draw.frag");

pub fn prepare(ctx: &Reference, state: &State) {
    // Changle the viewport.
    js! {
        @{ctx}.viewport(0, 0, @{WIDTH}, @{HEIGHT});
    }

    // Compile shaders.
    let prog: Reference = js! {
        var gl = @{ctx};

        var vertShader = gl.createShader(gl.VERTEX_SHADER);
        gl.shaderSource(vertShader, @{PLACE_VERT});
        gl.compileShader(vertShader);

        var vertShaderErr = gl.getShaderInfoLog(vertShader);
        if (vertShaderErr) console.error(vertShaderErr);

        var fragShader = gl.createShader(gl.FRAGMENT_SHADER);
        gl.shaderSource(fragShader, @{DRAW_FRAG});
        gl.compileShader(fragShader);

        var fragShaderErr = gl.getShaderInfoLog(fragShader);
        if (fragShaderErr) console.error(fragShaderErr);

        var program = gl.createProgram();
        gl.attachShader(program, vertShader);
        gl.attachShader(program, fragShader);
        gl.linkProgram(program);
        gl.useProgram(program);

        var progErr = gl.getProgramInfoLog(program);
        if (progErr) console.error(progErr);

        // Wrap the program to avoid TypeError.
        return [program];
    }.try_into().unwrap();

    // Setup a_tex_coords attribute.

    let item_count = state.map.len() + 2;

    let tex_coords_buf: Vec<_> = TEX_COORDS.iter().cycle().take(12 * item_count).collect();
    assert_eq!(tex_coords_buf.len(), 12 * item_count);

    js! {
        var gl = @{ctx};

        var texCoordsBuf = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, texCoordsBuf);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(@{tex_coords_buf}), gl.STATIC_DRAW);

        var texCoordsVbo = gl.getAttribLocation(@{&prog}[0], "a_tex_coords");
        gl.vertexAttribPointer(texCoordsVbo, 2, gl.FLOAT, false, 0, 0);
        gl.enableVertexAttribArray(texCoordsVbo);
    }

    // Setup a_tex_index attribute.

    let mut tex_index_buf = vec![0., 0., 0., 0., 0., 0., 1., 1., 1., 1., 1., 1.];
    tex_index_buf.extend(iter::repeat(0.).take(6 * (item_count - 2)));
    assert_eq!(tex_index_buf.len(), 6 * item_count);

    js! {
        var gl = @{ctx};

        var texIndexBuf = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, texIndexBuf);
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(@{tex_index_buf}), gl.STATIC_DRAW);

        var texIndexVbo = gl.getAttribLocation(@{&prog}[0], "a_tex_index");
        gl.vertexAttribPointer(texIndexVbo, 1, gl.FLOAT, false, 0, 0);
        gl.enableVertexAttribArray(texIndexVbo);
    }

    // Setup a_position attibute.
    js! {
        var gl = @{ctx};

        var positionBuf = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, positionBuf);

        var positionVbo = gl.getAttribLocation(@{&prog}[0], "a_position");
        gl.vertexAttribPointer(positionVbo, 2, gl.FLOAT, false, 0, 0);
        gl.enableVertexAttribArray(positionVbo);
    }

    // Setup textures.

    let rect_tex = generate_rect_texture();
    let ball_tex = generate_ball_texture();

    js! {
        var gl = @{ctx};

        var rectTex = gl.createTexture();
        gl.activeTexture(gl.TEXTURE0);
        gl.bindTexture(gl.TEXTURE_2D, rectTex);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, @{rect_tex}.canvas);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.uniform1i(gl.getUniformLocation(@{&prog}[0], "textures[0]"), 0);

        var ballTex = gl.createTexture();
        gl.activeTexture(gl.TEXTURE1);
        gl.bindTexture(gl.TEXTURE_2D, ballTex);
        gl.texImage2D(gl.TEXTURE_2D, 0, gl.RGBA, gl.RGBA, gl.UNSIGNED_BYTE, @{ball_tex}.canvas);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.LINEAR);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
        gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
        gl.uniform1i(gl.getUniformLocation(@{&prog}[0], "textures[1]"), 1);
    }

    // Setup uniforms.

    js! {
        var gl = @{ctx};

        gl.uniform2f(gl.getUniformLocation(@{&prog}[0], "u_shape"), @{WIDTH}, @{HEIGHT});
    }

    // Enable blending.

    js! {
        var gl = @{ctx};

        gl.enable(gl.BLEND);
        gl.blendFunc(gl.SRC_ALPHA, gl.DST_ALPHA);
    }
}

fn generate_rect_texture() -> CanvasRenderingContext2d {
    let ctx = create_draft_ctx(16, 16);

    ctx.fill_rect(0., 0., 16., 16.);

    ctx
}

fn generate_ball_texture() -> CanvasRenderingContext2d {
    let size = 64;

    let ctx = create_draft_ctx(size, size);

    let c = 0.5 * size as f64;

    ctx.begin_path();
    ctx.arc(c, c, c - 4., 0., 2. * PI, false);
    ctx.set_line_width(8.);
    ctx.stroke();

    ctx
}

fn create_draft_ctx(width: u32, height: u32) -> CanvasRenderingContext2d {
    let canvas: CanvasElement = document().create_element("canvas").unwrap().try_into().unwrap();

    canvas.set_width(width);
    canvas.set_height(height);

    canvas.get_context().unwrap()
}

// RENDER

pub fn render(ctx: &Reference, state: &State) {
    let vertex_count = 6 * (state.map.len() + 2);

    let buffer = unsafe { UnsafeTypedArray::new(&state.buffer) };

    js! { @(no_return)
        var gl = @{ctx};

        gl.bufferData(gl.ARRAY_BUFFER, @{buffer}, gl.DYNAMIC_DRAW);
        gl.drawArrays(gl.TRIANGLES, 0, @{vertex_count as u32});
    }

    if let Status::Failed = state.status {
        // TODO
    }
}
