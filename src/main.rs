use glfw::{fail_on_errors, Context};
use glow::HasContext;

use chrono::NaiveTime;

use thiserror::Error;

use animation::{Animation, AnimationReq};
use cursor_renderer::CursorRenderer;
use glyph_cache::GlyphCache;
use glyph_renderer::GlyphRenderer;
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};

mod animation;
mod cursor_renderer;
mod ease;
mod gl_util;
mod glyph_cache;
mod glyph_renderer;

#[derive(Error, Debug)]
#[error("{0}")]
struct GlError(String);

const WINDOW_WIDTH: u32 = 1920 / 2;
const WINDOW_HEIGHT: u32 = 1080 / 2;
const WINDOW_ASPECT: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32;

struct Args {
    start_time: NaiveTime,
    topic: String,
}

impl Args {
    fn parse<It: Iterator<Item = String>>(mut args: It) -> Args {
        let mut start_time = None;
        let mut topic = None;
        let process_name = args.next().unwrap_or_else(|| "prog".to_string());

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--start-time" => {
                    start_time = args.next().map(|v| v.parse());
                }
                "--topic" => {
                    topic = args.next();
                }
                _ => {
                    Self::help(&process_name);
                }
            }
        }

        let start_time = match start_time {
            Some(Ok(start_time)) => start_time,
            Some(Err(e)) => {
                println!("Failed to parse start time: {e}");
                Self::help(&process_name);
            }
            None => {
                println!("Start time not provided");
                Self::help(&process_name);
            }
        };

        let topic = match topic {
            Some(v) => v,
            None => {
                println!("Topic not provided");
                Self::help(&process_name);
            }
        };

        Args { start_time, topic }
    }

    fn help(process_name: &str) -> ! {
        println!(
            "\
                 A pre-stream screen...\n\
                 \n\
                 Usage:\n\
                 {process_name} [args]\n\
                 \n\
                 Arguments:\n\
                 --start-time: when stream starts\n\
                 --topic: what are we working on today\n\
                 "
        );
        std::process::exit(1);
    }
}

fn stream_starting_string(start_time: NaiveTime, now: NaiveTime, topic: &str) -> String {
    let remaining = start_time - now;
    let program = std::env::args().next().unwrap();
    format!(
        "\
        $ ./{}\n\
        \n\
        Today's topic: {}\n\
        Stream starting at {}\n\
            Current time: {}\n\
            {:02}:{:02}:{:02} 'till stream starts",
        program,
        topic,
        start_time.format("%H:%M:%S"),
        now.format("%H:%M:%S"),
        remaining.num_hours(),
        remaining.num_minutes() % 60,
        remaining.num_seconds() % 60,
    )
}

fn reset_animation(
    start_time: NaiveTime,
    topic: &str,
    current: String,
) -> (Animation, VecDeque<AnimationReq>) {
    let new_s = stream_starting_string(start_time, chrono::Local::now().time(), topic);
    let reqs = animation::construct_animation_requests(&current, &new_s);
    (Animation::None(current), reqs)
}

fn init_gl(window: &mut glfw::PWindow) -> glow::Context {
    unsafe {
        let gl = glow::Context::from_loader_function(|s| window.get_proc_address(s) as *const _);

        let r = 29.0f32 / 255.0f32;
        let g = 31.0f32 / 255.0f32;
        let b = 33.0f32 / 255.0f32;
        gl.clear_color(r, g, b, 1.0);

        gl.pixel_store_i32(glow::UNPACK_ALIGNMENT, 1);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        gl.enable(glow::BLEND);

        gl
    }
}

struct App<'a> {
    args: &'a Args,
    glyph_renderer: GlyphRenderer<'a>,
    cursor_renderer: CursorRenderer<'a>,
    current_animation: Animation,
    animation_queue: VecDeque<AnimationReq>,
    cursor_visible: bool,
    cursor_flip_time: Instant,
    cursor_blink_duration: Duration,
}

impl App<'_> {
    fn new<'a>(
        gl: &'a glow::Context,
        args: &'a Args,
        glyph_cache: &'a mut GlyphCache,
    ) -> Result<App<'a>, MainError> {
        let renderer =
            GlyphRenderer::new(gl, glyph_cache).map_err(MainError::CreateGlyphRenderer)?;
        let cursor_renderer = CursorRenderer::new(gl).unwrap();

        let (current_animation, animation_queue) =
            reset_animation(args.start_time, &args.topic, "".to_string());
        let cursor_visible = false;

        let cursor_blink_duration: Duration = Duration::from_secs_f32(0.5);
        let cursor_flip_time = Instant::now() + cursor_blink_duration;
        Ok(App {
            args,
            glyph_renderer: renderer,
            cursor_renderer,
            current_animation,
            animation_queue,
            cursor_visible,
            cursor_flip_time,
            cursor_blink_duration,
        })
    }

    fn update(&mut self, now: Instant) {
        if self.current_animation.finished(now) {
            let animation =
                std::mem::replace(&mut self.current_animation, Animation::None("".to_string()));
            let s = animation.into_finished_string();

            self.current_animation = match self.animation_queue.pop_front() {
                Some(req) => animation::apply_animation_req(req, s, now),
                None => {
                    (self.current_animation, self.animation_queue) =
                        reset_animation(self.args.start_time, &self.args.topic, s);
                    return;
                }
            }
        }

        self.current_animation.update(now);
    }

    fn render(&mut self, now: Instant) {
        let s = self.current_animation.as_str();

        let mut cursor_pos_x = 0.05;
        let mut cursor_pos_y = 0.7;
        let cursor_update =
            self.glyph_renderer
                .render_str(s, cursor_pos_x, cursor_pos_y, WINDOW_ASPECT);

        cursor_pos_x += cursor_update.0;
        cursor_pos_y += cursor_update.1;

        if self.cursor_flip_time < now {
            self.cursor_flip_time += self.cursor_blink_duration;
            self.cursor_visible = !self.cursor_visible;
        }

        if self.cursor_visible {
            let cursor_height = self.glyph_renderer.line_height() * 0.6;
            let cursor_width = cursor_height / 2.0;
            self.cursor_renderer.render(
                cursor_pos_x,
                cursor_pos_y,
                cursor_width,
                cursor_height,
                WINDOW_ASPECT,
            );
        }
    }
}

#[derive(Error, Debug)]
enum MainError {
    #[error("failed to init glfw")]
    InitGlfw(#[from] glfw::InitError),
    #[error("failed to create glfw window")]
    CreateGlfwWindow,
    #[error("failed to create glyph cache")]
    CreateGlyphCache(#[from] glyph_cache::GlyphCacheCreationError),
    #[error("failed to create glyph renderer")]
    CreateGlyphRenderer(GlError),
    #[error("failed to get character")]
    GetCharacter(#[from] glyph_cache::GetCharacterError),
}

fn main() -> Result<(), MainError> {
    let args = Args::parse(std::env::args());

    let mut glfw = glfw::init(fail_on_errors!())?;

    let (mut window, events) = glfw
        .create_window(
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            "Stream starting...",
            glfw::WindowMode::Windowed,
        )
        .ok_or(MainError::CreateGlfwWindow)?;

    window.make_current();
    window.set_key_polling(true);

    const PIXEL_SIZE: u32 = 256;
    let mut glyph_cache = GlyphCache::new(PIXEL_SIZE)?;
    let gl = init_gl(&mut window);

    let mut app = App::new(&gl, &args, &mut glyph_cache)?;

    while !window.should_close() {
        unsafe { gl.clear(glow::COLOR_BUFFER_BIT) };

        let now = Instant::now();
        app.update(now);
        app.render(now);

        window.swap_buffers();

        glfw.poll_events();
        for _ in glfw::flush_messages(&events) {}
    }

    Ok(())
}
