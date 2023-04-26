use ndarray::Array2;

#[derive(Debug, Clone)]
pub struct AppState {
    pub cycle: usize,
    pub objf: f64,
    pub theta: Array2<f64>,
    pub conv : bool,
}
impl AppState {
    pub fn new() -> Self {
        Self {
            cycle: 0,
            objf: f64::INFINITY,
            theta: Array2::default((0, 0)),
            conv : false,
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
