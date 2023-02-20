// use std::array;

// use ndarray::array;
// use ndarray::{OwnedRepr, Zip};
// use plotly::{Plot, Scatter};
use ode_solvers::*;
// extern crate blas_src;
use np_core::prelude::*;
// use ndarray::parallel::prelude::*;
// use ndarray_stats::{QuantileExt, DeviationExt};
// use ndarray::{prelude::*, parallel::prelude::{IntoParallelIterator, IntoParallelRefIterator, IntoParallelRefMutIterator}};

struct Model<'a>{
    ka: f64,
    ke: f64,
    _v: f64,
    lag: f64,
    scenario: &'a Scenario
}

type State = Vector2<f64>;
type Time = f64;

impl ode_solvers::System<State> for Model<'_> {
    fn system(&self, t: Time, y: &mut State, dy: &mut State) {

        let ka = self.ka;
        let ke = self.ke;
        let t = t - self.lag;
        
        ///////////////////// USER DEFINED ///////////////

        dy[0] = -ka*y[0];
        dy[1] = ka*y[0] - ke*y[1];

        //////////////// END USER DEFINED ////////////////
        for index in 0..self.scenario.dose.len(){
            if (t-self.scenario.time_dose[index] as f64).abs() < 1.0e-4 {
                y[0] = y[0]+self.scenario.dose[index].0 as f64;
            }
        }

    }
}

struct Sim{}

impl Simulate for Sim{
    fn simulate(&self, params: Vec<f64>, tspan:[f64;2], scenario: &Scenario) -> (Vec<f64>, Vec<f64>) {
        let system = Model {ka: params[0], ke: params[1], _v: params[2], lag: params[3], scenario: scenario};
    
        let y0 = State::new(0.0, 0.0);
    
        let mut stepper = Rk4::new(system, tspan[0], y0, tspan[1],0.1);
        // let mut stepper = Dopri5::new(system, 0.0, 20.0, 1.0e-5, y0, 1.0e-14, 1.0e-14);
        let _res = stepper.integrate();
        let x = stepper.x_out().to_vec();
        let y = stepper.y_out();
        
        let yout: Vec<f64> = y.into_iter().map(|y| {
            y[1]/params[2]
        } ).collect();

  
        (x, yout)    
    }
} 

fn main(){
    let engine = Engine::new(Sim{});
    npag(engine,
        vec![(0.1,0.9),(0.001,0.1),(30.0,120.0),(0.0,4.0)],
        "examples/two_eq_lag.toml".to_string(),
        347,
        (0.5,0.1,0.0,0.0)
    ); 
}