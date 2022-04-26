use speedy2d::dimen::Vector2;
use speedy2d::{Window, Graphics2D};
use speedy2d::window::{WindowHelper, WindowHandler, MouseButton};
use speedy2d::color::Color;

extern crate nalgebra as na;
use na::{Const, Dynamic, ArrayStorage, VecStorage, Matrix, DMatrix, RealField};
use rand::{thread_rng, Rng};

type MatDyn = Matrix<f32, Dynamic, Dynamic, VecStorage<f32, Dynamic, Dynamic>>;
type VecDyn = Matrix<f32, Const<1>, Dynamic, VecStorage<f32, Const<1>, Dynamic>>;
type Mat2D = Matrix<f32, Dynamic, Const<2>, VecStorage<f32, Dynamic, Const<2>>>;
type Vec2D = Matrix<f32, Const<1>, Const<2>, ArrayStorage<f32, 1, 2>>;
pub type VerticeLoc = (usize, usize);

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

    fn get_edg_value(&self, v0: VerticeLoc, v1: VerticeLoc) -> f32 {
        let row = self.idx(v0);
        let col = self.idx(v1);
        self.mat[(col, row)]
    }

    fn set_edg_value(&mut self, v0: VerticeLoc, v1: VerticeLoc, value: f32) {
        let row = self.idx(v0);
        let col = self.idx(v1);
        self.mat[(col, row)] = value;
    }

    fn idx(&self, vertice: VerticeLoc) -> usize {
        vertice.0 + vertice.1 * self.width
    }
}

pub struct ACOMap {
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
                aco_map.set_outgoing_costs((i, j));
            }
        }

        aco_map.pheromone_graph.mat.fill(1.0);
        
        return Some(aco_map);
    }

    /// Get the cost for traversing from vertice v0 to v1
    fn cost(v0: VerticeLoc, v1: VerticeLoc) -> f32 {
        const SQRT_OF_2: f32 = 1.41421356237;
        if v0.0 != v1.0 && v0.1 != v1.1 {
            SQRT_OF_2
        } else {
            1.0
        }
    }

    fn set_outgoing_costs(&mut self, vertice: VerticeLoc) {
        self.get_neighbours(vertice).iter().for_each(|neighbour| {
            self.cost_graph.set_edg_value(vertice, *neighbour, ACOMap::cost(vertice, *neighbour));
        });
    }

    fn get_neighbours(&self, vertice: VerticeLoc) -> Vec<VerticeLoc> {
        let mut neighbours: Vec<VerticeLoc> = Vec::new();
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

                neighbours.push((new_x as usize, new_y as usize));
            }
        }
        return neighbours;
    }

    fn get_neighbours_with_exclusions(&self, vertice: VerticeLoc, exclusions: &Vec<VerticeLoc>) -> Vec<VerticeLoc> {
        let mut neighbours: Vec<VerticeLoc> = Vec::new();
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

                let neighbour: VerticeLoc = (new_x as usize, new_y as usize);
                if !exclusions.contains(&neighbour) {
                    neighbours.push(neighbour);
                }
            }
        }
        return neighbours;
    }

    fn get_likelyhood_factor(&self, v0: VerticeLoc, v1: VerticeLoc) -> f32 {
        let pheromone = self.pheromone_graph.get_edg_value(v0, v1);
        let cost = ACOMap::cost(v0, v1);
        pheromone / cost
    }

    pub fn get_next_vertice(&self, current: VerticeLoc) -> Option<VerticeLoc> {
        let mut likelyhood_sum = 0.0;
        let mut neighbours: Vec<(f32, VerticeLoc)> = self.get_neighbours(current).iter().map(|neighbour| {
            let likelyhood = self.get_likelyhood_factor(current, *neighbour);
            likelyhood_sum += likelyhood;
            (likelyhood, *neighbour)
        }).collect();
        neighbours.iter_mut().for_each(|pair| pair.0 = pair.0 / likelyhood_sum);
        neighbours.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut probability_sum = 0.0;
        neighbours.iter_mut().for_each(|mut pair| {
            probability_sum += pair.0;
            pair.0 = probability_sum;
        });

        let mut rng = thread_rng();
        let random: f32 = rng.gen();
        let mut previous = 0.0;
        for pair in neighbours {
            if random >= previous && random < pair.0 {
                return Some(pair.1);
            } else {
                previous = pair.0;
            }
        }
        None
    }

    pub fn get_next_vertice_with_exclusions(&self, current: VerticeLoc, exclusions: &Vec<VerticeLoc>) -> Option<VerticeLoc> {
        let mut likelyhood_sum = 0.0;
        let mut neighbours: Vec<(f32, VerticeLoc)> = self.get_neighbours_with_exclusions(current, exclusions).iter().map(|neighbour| {
            let likelyhood = self.get_likelyhood_factor(current, *neighbour);
            likelyhood_sum += likelyhood;
            (likelyhood, *neighbour)
        }).collect();

        if neighbours.len() == 0 {
            return None;
        }
        
        neighbours.iter_mut().for_each(|pair| pair.0 = pair.0 / likelyhood_sum);
        neighbours.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
        let mut probability_sum = 0.0;
        neighbours.iter_mut().for_each(|mut pair| {
            probability_sum += pair.0;
            pair.0 = probability_sum;
        });

        let mut rng = thread_rng();
        let random: f32 = rng.gen();
        let mut previous = 0.0;
        for pair in neighbours {
            if random >= previous && random < pair.0 {
                return Some(pair.1);
            } else {
                previous = pair.0;
            }
        }
        None
    }

    fn find_path(v0: VerticeLoc, v1: VerticeLoc) -> Vec<VerticeLoc> {
        Vec::new()
    }

    pub fn render(&self, window_size: (usize, usize), graphics: &mut Graphics2D) {
        let x_spacing = window_size.0 as f32 / self.cost_graph.width as f32;
        let y_spacing = (window_size.1 as f32 - x_spacing) / (self.cost_graph.height - 1) as f32;
        let r = if x_spacing < y_spacing { x_spacing / 20.0 } else { y_spacing / 20.0 };
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

    pub fn get_vertice_coordinates(&self, window_size: (usize, usize), vertice: VerticeLoc) -> (f32, f32) {
        let x_spacing = window_size.0 as f32 / self.cost_graph.width as f32;
        let y_spacing = (window_size.1 as f32 - x_spacing) / (self.cost_graph.height - 1) as f32;
        let x_offs = x_spacing / 2.0;
        let y_offs = x_offs;
        let x = x_offs + vertice.0 as f32 * x_spacing;
        let y = y_offs + vertice.1 as f32 * y_spacing;
        (x, y)
    }
}
