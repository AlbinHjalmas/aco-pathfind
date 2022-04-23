use std::time::{Instant, Duration};
use std::string::String;

use speedy2d::dimen::Vector2;
use speedy2d::{Window, Graphics2D};
use speedy2d::window::{WindowHelper, WindowHandler, MouseButton};
use speedy2d::color::Color;

extern crate nalgebra as na;
use na::{Const, Dynamic, ArrayStorage, VecStorage, Matrix, DMatrix, RealField};

type MatDyn = Matrix<f32, Dynamic, Dynamic, VecStorage<f32, Dynamic, Dynamic>>;
type VecDyn = Matrix<f32, Const<1>, Dynamic, VecStorage<f32, Const<1>, Dynamic>>;
type Mat2D = Matrix<f32, Dynamic, Const<2>, VecStorage<f32, Dynamic, Const<2>>>;
type Vec2D = Matrix<f32, Const<1>, Const<2>, ArrayStorage<f32, 1, 2>>;
type VerticeIdx = (usize, usize);

trait Renderable<Graphics> {
    fn render(&self, window_size: (usize, usize), graphics: &mut Graphics);
}

impl Renderable<Graphics2D> for ACOMap {
    fn render(&self, window_size: (usize, usize), graphics: &mut Graphics2D) {
        let x_spacing = window_size.0 as f32 / self.cost_graph.width as f32;
        let y_spacing = (window_size.1 as f32 - x_spacing) / (self.cost_graph.height - 1) as f32;
        let r = if x_spacing < y_spacing { x_spacing / 6.0 } else { y_spacing / 6.0 };
        let x_offs = x_spacing / 2.0;
        let y_offs = x_offs;

        for i in 0..self.cost_graph.width {
            let x = x_offs + i as f32 * x_spacing;
            for j in 0..self.cost_graph.height {
                let y = y_offs + j as f32 * y_spacing;
                graphics.draw_circle((x, y), r, Color::GRAY);
            }
        }
    }
}
struct ACOGraph {
    mat: MatDyn,
    width: usize,
    height: usize
}

impl ACOGraph {
    fn new(width: usize, height: usize) -> Self {
        let n_vertices = width * height;
        ACOGraph {mat: MatDyn::from_diagonal_element(n_vertices, n_vertices, 0.0), width, height}
    }

    fn get_edg_value(&self, v0: VerticeIdx, v1: VerticeIdx) -> f32 {
        let row = self.idx(v0);
        let col = self.idx(v1);
        self.mat[(col, row)]
    }

    fn set_edg_value(&mut self, v0: VerticeIdx, v1: VerticeIdx, value: f32) {
        let row = self.idx(v0);
        let col = self.idx(v1);
        self.mat[(col, row)] = value;
    }

    fn idx(&self, vertice: VerticeIdx) -> usize {
        vertice.0 + vertice.1 * self.width
    }
}

struct ACOMap {
    cost_graph: ACOGraph,
    pheromone_graph: ACOGraph,
    evaporation_rate: f32
}

impl ACOMap {
    pub fn new(width: usize, height: usize, evaporation_rate: f32) -> Option<Self> {
        if width == 0 || height == 0 || evaporation_rate > 1.0 {
            return None;
        }

        let mut aco_map = ACOMap {
            cost_graph: ACOGraph::new(width, height),
            pheromone_graph: ACOGraph::new(width, height),
            evaporation_rate
        };

        for i in 0..width {
            for j in 0..height {
                let current_vertice = (i, j);
                aco_map.set_outgoing_costs(current_vertice);
            }
        }
        
        return Some(aco_map);
    }

    /// Get the cost for traversing from vertice (x_0, y_0) to (x_1, y_1)
    fn cost(v0: VerticeIdx, v1: VerticeIdx) -> f32 {
        const SQRT_OF_2: f32 = 1.41421356237;
        if v0.0 != v1.0 && v0.1 != v1.1 {
            SQRT_OF_2
        } else {
            1.0
        }
    }

    fn set_outgoing_costs(&mut self, vertice: VerticeIdx) {
        for i in &[-1, 0, 1] {
            let new_x = (vertice.0 as i32) + i;
            if new_x < 0 || new_x >= self.cost_graph.width as i32 {
                // Resulting vertice will be outside map
                continue;
            }
            for j in &[-1, 0, 1] {
                let new_y = (vertice.1 as i32) + j;
                if new_y < 0 || new_y >= self.cost_graph.height as i32 || (*i == 0 && *j == 0) {
                    // Resulting vertice will be outside map
                    continue;
                }

                let v1 = (new_x as usize, new_y as usize);
                self.cost_graph.set_edg_value(vertice, v1, ACOMap::cost(vertice, v1));
            }
        }
    }
}

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

    aco_map: ACOMap
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
        aco_map: ACOMap::new(15, 15, 0.5).expect("Failed to generate ACO map...")
    };

    window.run_loop(window_context);
}
