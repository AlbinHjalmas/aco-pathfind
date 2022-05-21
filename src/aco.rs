use speedy2d::Graphics2D;
use speedy2d::color::Color;

extern crate nalgebra as na;
use na::{Dynamic, VecStorage, Matrix};

type MatDyn = Matrix<f32, Dynamic, Dynamic, VecStorage<f32, Dynamic, Dynamic>>;
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

    #[allow(dead_code)]
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
    pheromone_graph: ACOGraph,
    _evaporation_rate: f32
}

impl ACOMap {
    #[allow(dead_code)]
    pub fn new(width: usize, height: usize, evaporation_rate: f32) -> Option<Self> {
        if width == 0 || height == 0 || evaporation_rate > 1.0 {
            return None;
        }
        let mut aco_map = ACOMap {
            pheromone_graph: ACOGraph::new(width, height),
            _evaporation_rate: evaporation_rate
        };
        aco_map.pheromone_graph.mat.fill(1.0);
        return Some(aco_map);
    }

    /// Get the cost for traversing from vertice v0 to v1
    #[allow(dead_code)]
    fn cost(v0: VerticeLoc, v1: VerticeLoc) -> f32 {
        const SQRT_OF_2: f32 = 1.41421356237;
        if v0.0 != v1.0 && v0.1 != v1.1 {
            SQRT_OF_2
        } else {
            1.0
        }
    }

    #[allow(dead_code)]
    fn get_neighbours(&self, vertice: VerticeLoc) -> Vec<VerticeLoc> {
        let mut neighbours: Vec<VerticeLoc> = Vec::new();
        for i in &[-1, 0, 1] {
            let new_x = (vertice.0 as i32) + i;
            if new_x < 0 || new_x >= self.pheromone_graph.width as i32 {
                // Resulting vertice will be outside map
                continue;
            }
            for j in &[-1, 0, 1] {
                let new_y = (vertice.1 as i32) + j;
                if new_y < 0 || new_y >= self.pheromone_graph.height as i32 || (*i == 0 && *j == 0) {
                    // Resulting vertice will be outside map
                    continue;
                }

                neighbours.push((new_x as usize, new_y as usize));
            }
        }
        return neighbours;
    }

    #[allow(dead_code)]
    fn get_neighbours_with_exclusions(&self, vertice: VerticeLoc, exclusions: &Vec<VerticeLoc>) -> Vec<VerticeLoc> {
        let mut neighbours: Vec<VerticeLoc> = Vec::new();
        for i in &[-1, 0, 1] {
            let new_x = (vertice.0 as i32) + i;
            if new_x < 0 || new_x >= self.pheromone_graph.width as i32 {
                // Resulting vertice will be outside map
                continue;
            }
            for j in &[-1, 0, 1] {
                let new_y = (vertice.1 as i32) + j;
                if new_y < 0 || new_y >= self.pheromone_graph.height as i32 || (*i == 0 && *j == 0) {
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

    #[allow(dead_code)]
    pub fn get_next_vertice(&self, current: VerticeLoc) -> Option<VerticeLoc> {
        let mut likelyhood_sum = 0.0;

        use crate::roulette::RouletteSubjects;
        let mut neighbours = RouletteSubjects::<VerticeLoc>(
            self.get_neighbours(current)
                .iter()
                .map(|neighbour| {
                    let likelyhood = self.get_likelyhood_factor(current, *neighbour);
                    likelyhood_sum += likelyhood;
                    (likelyhood, *neighbour)
                })
                .collect()
        );

        if neighbours.len() == 0 {
            return None
        }

        neighbours.iter_mut().for_each(|pair| {pair.0 = pair.0 / likelyhood_sum});
        neighbours.roulette()
    }

    #[allow(dead_code)]
    pub fn get_next_vertice_with_exclusions(&self, current: VerticeLoc, exclusions: &Vec<VerticeLoc>) -> Option<VerticeLoc> {
        use crate::roulette::RouletteSubjects;
        let mut likelyhood_sum = 0.0;
        let mut neighbours = RouletteSubjects::<VerticeLoc>(
            self.get_neighbours_with_exclusions(current, exclusions)
                .iter()
                .map(|neighbour| {
                    let likelyhood = self.get_likelyhood_factor(current, *neighbour);
                    likelyhood_sum += likelyhood;
                    (likelyhood, *neighbour)
                })
                .collect() 
        );

        if neighbours.len() == 0 {
            return None;
        }
        
        neighbours.iter_mut().for_each(|pair| pair.0 = pair.0 / likelyhood_sum);
        neighbours.roulette()
    }

    #[allow(dead_code)]
    fn find_path(_v0: VerticeLoc, _v1: VerticeLoc) -> Vec<VerticeLoc> {
        Vec::new()
    }

    #[allow(dead_code)]
    pub fn render(&self, window_size: (usize, usize), graphics: &mut Graphics2D) {
        let x_spacing = window_size.0 as f32 / self.pheromone_graph.width as f32;
        let y_spacing = (window_size.1 as f32 - x_spacing) / (self.pheromone_graph.height - 1) as f32;
        let r = if x_spacing < y_spacing { x_spacing / 20.0 } else { y_spacing / 20.0 };
        let x_offs = x_spacing / 2.0;
        let y_offs = x_offs;

        for i in 0..self.pheromone_graph.width {
            let x = x_offs + i as f32 * x_spacing;
            for j in 0..self.pheromone_graph.height {
                let y = y_offs + j as f32 * y_spacing;
                graphics.draw_circle((x, y), r, Color::GRAY);
            }
        }
    }

    #[allow(dead_code)]
    pub fn get_vertice_coordinates(&self, window_size: (usize, usize), vertice: VerticeLoc) -> (f32, f32) {
        let x_spacing = window_size.0 as f32 / self.pheromone_graph.width as f32;
        let y_spacing = (window_size.1 as f32 - x_spacing) / (self.pheromone_graph.height - 1) as f32;
        let x_offs = x_spacing / 2.0;
        let y_offs = x_offs;
        let x = x_offs + vertice.0 as f32 * x_spacing;
        let y = y_offs + vertice.1 as f32 * y_spacing;
        (x, y)
    }
}
