mod aco;
use aco::{ACOMap, VerticeLoc};

use std::time::{Instant, Duration};

use speedy2d::dimen::Vector2;
use speedy2d::{Window, Graphics2D};
use speedy2d::window::{WindowHelper, WindowHandler, MouseButton};
use speedy2d::color::Color;

struct PointerStatus {
    position: (f32, f32),
    l_btn_pushed: bool,
    r_btn_pushed: bool
}

impl PointerStatus {
    fn new() -> PointerStatus {
        PointerStatus {position: (0.0, 0.0), l_btn_pushed: false, r_btn_pushed: false}
    }
}

struct WindowContext {
    pointer_status: PointerStatus,
    window_size: (usize, usize),
    prev_time: Instant,
    accumulated_duration: Duration,
    accumulated_interpolation_duration: Duration,
    iterations: usize,

    aco_map: ACOMap,
    curr_vert: VerticeLoc,
    path: Vec<VerticeLoc>,
    exclusions: Vec<VerticeLoc>
}

impl WindowHandler for WindowContext {
    fn on_draw(&mut self, helper: &mut WindowHelper<()>, graphics: &mut Graphics2D)
    {
        graphics.clear_screen(Color::WHITE);

        let curr_time = std::time::Instant::now();
        let duration = curr_time.duration_since(self.prev_time);
        
        if self.iterations % 100 == 0 {
            let avg_frame_rate = self.iterations as f64 / self.accumulated_duration.as_secs_f64();
            println!("Framerate: {}", avg_frame_rate);
        }

        self.aco_map.render(self.window_size, graphics);
        // if self.iterations % 5 == 0 {
            let mut got_next = false;
            while got_next == false {
                match self.aco_map.get_next_vertice_with_exclusions(
                self.curr_vert, &[self.path.as_slice(), self.exclusions.as_slice()].concat()) {
                    None => {
                        if self.exclusions.len() > 150 {
                            self.exclusions.remove(0);
                        }
                        self.exclusions.push(self.curr_vert);

                        self.curr_vert = self.path.pop().unwrap();
                        got_next = false;
                    },
                    Some(next_vertice) => {
                        self.path.push(self.curr_vert);
                        self.curr_vert = next_vertice;
                        got_next = true;
                    }
                };
            }
        // }
        self.path.windows(2).for_each(|points| {
            graphics.draw_line(
                self.aco_map.get_vertice_coordinates(self.window_size, points[0]), 
                self.aco_map.get_vertice_coordinates(self.window_size, points[1]),
                1.0, 
                Color::GREEN
            );
        });
        graphics.draw_line(
            self.aco_map.get_vertice_coordinates(self.window_size, *self.path.last().unwrap()), 
            self.aco_map.get_vertice_coordinates(self.window_size, self.curr_vert),
            1.0, 
            Color::GREEN
        );
        graphics.draw_circle(self.aco_map.get_vertice_coordinates(self.window_size, 
            self.curr_vert), 4.0, Color::RED);


        // Store the time to be able to measure duration
        self.iterations += 1;
        self.prev_time = curr_time;
        self.accumulated_duration += duration;
        // Request that we draw another frame once this one has finished
        helper.request_redraw();
    }

    fn on_mouse_move(&mut self, helper: &mut WindowHelper<()>, position: Vector2<f32>) {
        self.pointer_status.position = (position.x, position.y);
    }

    fn on_mouse_button_down(&mut self, helper: &mut WindowHelper<()>, button: MouseButton) {
        match button {
            MouseButton::Left => self.pointer_status.l_btn_pushed = true,
            MouseButton::Right => self.pointer_status.r_btn_pushed = true,
            _ => ()
        }
    }

    fn on_mouse_button_up(&mut self, helper: &mut WindowHelper<()>, button: speedy2d::window::MouseButton) {
        match button {
            MouseButton::Left => self.pointer_status.l_btn_pushed = false,
            MouseButton::Right => self.pointer_status.r_btn_pushed = false,
            _ => return
        }
    }
}

fn main() {
    let window = Window::new_centered("Abbes testfönster <3", (1200, 1200)).unwrap();
    let mut window_context = WindowContext {
        pointer_status: PointerStatus::new(),
        window_size: (1200, 1200),
        prev_time: Instant::now(),
        accumulated_duration: Duration::new(0, 0),
        accumulated_interpolation_duration: Duration::new(0, 0),
        iterations: 0,
        aco_map: ACOMap::new(100, 100, 0.5).expect("Failed to generate ACO map..."),
        curr_vert: (7, 7),
        path: Vec::new(),
        exclusions: Vec::new()
    };
    window_context.path.push(window_context.curr_vert);

    window.run_loop(window_context);
}
