#[macro_use]
extern crate stdweb;

use std::rc::Rc;
use std::cell::Cell;

use stdweb::web;
use stdweb::unstable::TryInto;
use stdweb::web::{IParentNode, INode, IEventTarget};
use stdweb::web::event::{KeyDownEvent, KeyUpEvent, IKeyboardEvent};
use stdweb::web::html_element::CanvasElement;

use game::{State, Input, Context};

mod game;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 800;

fn timer(state: State, ctx: Context, left: Rc<Cell<bool>>, right: Rc<Cell<bool>>, prev_ts: f64) {
    let window = web::window();

    window.request_animation_frame(move |ts| {
        let dt = if prev_ts == 0. { 0. } else { ts - prev_ts };

        let state = game::simulate(state, Input {
            dt: dt * 0.001,
            left: left.get(),
            right: right.get()
        });

        game::render(&ctx, &state);

        timer(state, ctx, left, right, ts);
    });
}

macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

fn keyboard(left: Rc<Cell<bool>>, right: Rc<Cell<bool>>) {
    let window = web::window();

    window.add_event_listener(enclose!( (left, right) move |ev: KeyDownEvent| {
        match ev.key().as_str() {
            "ArrowRight" => right.set(true),
            "ArrowLeft" => left.set(true),
            _ => {},
        };
    }));

    window.add_event_listener(enclose!( (left, right) move |ev: KeyUpEvent| {
        match ev.key().as_str() {
            "ArrowRight" => right.set(false),
            "ArrowLeft" => left.set(false),
            _ => {},
        };
    }));
}

fn main() {
    stdweb::initialize();

    let document = web::document();

    let canvas: CanvasElement = document.create_element("canvas").unwrap().try_into().unwrap();
    let body = document.query_selector("body").unwrap().unwrap();

    body.append_child(&canvas);

    canvas.set_width(WIDTH);
    canvas.set_height(HEIGHT);

    let state = State::new();
    let left = Rc::new(Cell::new(false));
    let right = Rc::new(Cell::new(false));

    let ctx = Context {
        ctx: canvas.get_context().unwrap(),
        width: WIDTH as f64,
        height: HEIGHT as f64,
    };

    timer(state, ctx, left.clone(), right.clone(), 0.);
    keyboard(left, right);

    stdweb::event_loop();
}
