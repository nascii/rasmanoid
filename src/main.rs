#![recursion_limit="128"]

#[macro_use]
extern crate stdweb;

use std::rc::Rc;
use std::cell::RefCell;

use stdweb::web;
use stdweb::unstable::TryInto;
use stdweb::web::{IParentNode, INode, IEventTarget, CanvasRenderingContext2d};
use stdweb::web::event::{KeyDownEvent, KeyUpEvent, IKeyboardEvent};
use stdweb::web::html_element::CanvasElement;

use game::{State, Input};

mod game;

const WIDTH: u32 = 600;
const HEIGHT: u32 = 800;

struct EnvInner {
    left: bool,
    right: bool,
    timestamp: f64,
}

type Env = Rc<RefCell<EnvInner>>;

fn timer(state: State, ctx: CanvasRenderingContext2d, env: Env) {
    let window = web::window();

    window.request_animation_frame(move |ts| {
        let state = {
            let mut e = env.borrow_mut();

            let dt = if e.timestamp == 0. { 0. } else { ts - e.timestamp };

            e.timestamp = ts;

            let state = game::simulate(state, Input {
                dt: dt * 0.001,
                left: e.left,
                right: e.right,
            });

            game::render(&ctx, &state);

            state
        };

        timer(state, ctx, env);
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

fn keyboard(env: Env) {
    let window = web::window();

    window.add_event_listener(enclose!( (env) move |ev: KeyDownEvent| {
        match ev.key().as_str() {
            "ArrowRight" => env.borrow_mut().right = true,
            "ArrowLeft" => env.borrow_mut().left = true,
            _ => {},
        };
    }));

    window.add_event_listener(enclose!( (env) move |ev: KeyUpEvent| {
        match ev.key().as_str() {
            "ArrowRight" => env.borrow_mut().right = false,
            "ArrowLeft" => env.borrow_mut().left = false,
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

    let ctx: CanvasRenderingContext2d = canvas.get_context().unwrap();

    js! {
        @{canvas}.style = "border: 1px solid gray";
    }

    let state = State::new(WIDTH, HEIGHT);

    let env = Rc::new(RefCell::new(EnvInner {
        left: false,
        right: false,
        timestamp: 0.,
    }));

    timer(state, ctx, env.clone());
    keyboard(env);

    stdweb::event_loop();
}
