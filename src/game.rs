use stdweb::web::{CanvasRenderingContext2d};

const BAT_WIDTH_DEFAULT: f64 = 0.3;

pub struct State {
    bat_x: f64,
    bat_v: f64,
    bat_w: f64,
}

impl State {
    pub fn new() -> State {
        State {
            bat_x: 0.5,
            bat_v: 0.,
            bat_w: BAT_WIDTH_DEFAULT,
        }
    }
}

pub struct Input {
    pub dt: f64,
    pub left: bool,
    pub right: bool,
}

pub struct Context {
    pub ctx: CanvasRenderingContext2d,
    pub width: f64,
    pub height: f64,
}

// SIMULATE

pub fn simulate(state: State, input: Input) -> State {
    let dv = if input.left { -0.02 } else if input.right { 0.02 } else { 0. };

    let dx = state.bat_v * input.dt;

    State {
        bat_x: (0.5 * state.bat_w).max(state.bat_x + dx).min(1. - 0.5 * state.bat_w),
        bat_v: 0.98 * (state.bat_v + dv),
        bat_w: state.bat_w,
    }
}

// RENDER

pub fn render(context: &Context, state: &State) {
    clear(context);
    draw_rect(context, state.bat_x, 0.1, state.bat_w, 0.1);
    draw_circle(context, 0.5, 0.5, 0.05);
}

fn clear(context: &Context) {
    let ctx = &context.ctx;

    js! { @{ctx}.clearRect(0, 0, @{context.width}, @{context.height}) }
}

fn draw_rect(context: &Context, x: f64, y: f64, w: f64, h: f64) {
    let hw = 0.5 * w;
    let hh = 0.5 * h;

    context.ctx.fill_rect(
        (x - hw) * context.width,
        (1. - y + hh) * context.height,
        w * context.width,
        h * context.height
    );
}

fn draw_circle(context: &Context, x: f64, y: f64, r: f64) {
    let ctx = &context.ctx;

    let x = x * context.width;
    let y = y * context.height;
    let r = r * context.width;

    js! {
        @{ctx}.beginPath();
        @{ctx}.arc(@{x}, @{y}, @{r}, 0, 2 * Math.PI, false);
        @{ctx}.lineWidth = 2;
        @{ctx}.stroke();
    }
}
