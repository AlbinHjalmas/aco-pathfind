use rand::{thread_rng, Rng};

pub struct RouletteSubjects<T> (pub Vec<(f32, T)>);

impl<T> RouletteSubjects<T> 
where T: Copy,
{
    #[allow(dead_code)]
    pub fn new() -> Self {
        RouletteSubjects::<T>(Vec::new())
    }

    pub fn roulette(&mut self) -> Option<T> {
        self.sort();
        let mut probability_sum = 0.0;
        self.iter_mut().for_each(|mut pair| {
            probability_sum += pair.0;
            pair.0 = probability_sum;
        });

        let mut rng = thread_rng();
        let random: f32 = rng.gen::<f32>() * probability_sum;
        let mut previous = 0.0;


        for pair in &self.0 {
            if random >= previous && random < pair.0 {
                return Some((*pair).1);
            } else {
                previous = pair.0;
            }
        }

        None
    }

    #[inline(always)]
    fn sort(&mut self) {
        self.0.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline(always)]
    #[allow(dead_code)]
    pub fn push(&mut self, value: (f32, T)) {
        self.0.push(value);
    }

    #[inline(always)]
    pub fn iter_mut(&mut self) -> impl Iterator<Item=&mut (f32, T)> {
        self.0.iter_mut()
    }
}

#[test]
fn test_vertice_probabilities_sort() {
    let mut probabilities = RouletteSubjects::new();
    probabilities.push((0.5, (5, 0)));
    probabilities.push((0.2, (2, 0)));
    probabilities.push((0.3, (3, 0)));
    assert_eq!(probabilities.0, vec![(0.5, (5, 0)), (0.2, (2, 0)), (0.3, (3, 0))]);
    probabilities.sort();
    assert_eq!(probabilities.0, vec![(0.2, (2, 0)), (0.3, (3, 0)), (0.5, (5, 0))]);
    probabilities.sort();
    assert_eq!(probabilities.0, vec![(0.2, (2, 0)), (0.3, (3, 0)), (0.5, (5, 0))]);
}

#[test]
fn test_vertice_probabilities_roulette() {
    let mut probabilities = RouletteSubjects::new();
    probabilities.push((0.5, (5, 0)));
    probabilities.push((0.2, (2, 0)));
    probabilities.push((0.3, (3, 0)));

    let mut cnt_02 = 0;
    let mut cnt_03 = 0;
    let mut cnt_05 = 0;

    const ITERATIONS: usize = 1000000;

    (0..ITERATIONS).into_iter().for_each(|_| {
        match probabilities.roulette() {
            Some(v) => {
                match v.0 {
                    2 => cnt_02 += 1,
                    3 => cnt_03 += 1,
                    5 => cnt_05 += 1,
                    _ => ()
                }
            },
            None => ()
        }
    });

    let frq_02 = (cnt_02 as f32 / ITERATIONS as f32) * 10.0;
    let frq_03 = (cnt_03 as f32 / ITERATIONS as f32) * 10.0;
    let frq_05 = (cnt_05 as f32 / ITERATIONS as f32) * 10.0;

    assert_eq!(frq_02.round() as u32, 2);
    assert_eq!(frq_03.round() as u32, 3);
    assert_eq!(frq_05.round() as u32, 5);

    println!("freq(0.2) = {}, freq(0.3) = {}, freq(0.5) = {}", frq_02.round() as u32, frq_03.round() as u32, frq_05.round() as u32);
}