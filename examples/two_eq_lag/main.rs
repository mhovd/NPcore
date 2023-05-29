use std::collections::HashMap;

use eyre::Result;
use np_core::prelude::{
    datafile::{CovLine, Dose, Infusion},
    *,
};
use ode_solvers::*;
#[derive(Debug, Clone)]
struct Model<'a> {
    ka: f64,
    ke: f64,
    _v: f64,
    lag: f64,
    _scenario: &'a Scenario,
    infusions: Vec<Infusion>,
    dose: Option<Dose>,
    cov: Option<&'a HashMap<String, CovLine>>,
}

type State = Vector2<f64>;
type Time = f64;

impl ode_solvers::System<State> for Model<'_> {
    fn system(&mut self, t: Time, y: &mut State, dy: &mut State) {
        // Random parameters
        let ka = self.ka;
        let ke = self.ke;
        // Covariates
        let _wt = self.cov.unwrap().get("WT").unwrap().interp(t);
        ///////////////////// USER DEFINED ///////////////
        dy[0] = -ka * y[0];
        dy[1] = ka * y[0] - ke * y[1];
        //////////////// END USER DEFINED ////////////////

        if let Some(dose) = &self.dose {
            if dose.time > t - (0.1 / 2.) && dose.time <= t + (0.1 / 2.) {
                y[dose.compartment] += dose.amount;
            }
        }
    }
}

struct Ode {}

impl Predict for Ode {
    fn predict(&self, params: Vec<f64>, scenario: &Scenario) -> Vec<f64> {
        let mut system = Model {
            ka: params[0],
            ke: params[1],
            _v: params[2],
            lag: params[3],
            _scenario: scenario,
            infusions: vec![],
            dose: None,
            cov: None,
        };
        let lag = system.lag; // or 0.0
        let mut yout = vec![];
        let mut y0 = State::new(0.0, 0.0);
        let mut index = 0;
        for block in &scenario.blocks {
            system.cov = Some(&block.covs);
            for event in &block.events {
                if event.evid == 1 {
                    if event.dur.unwrap_or(0.0) > 0.0 {
                        //infusion
                        system.infusions.push(Infusion {
                            time: event.time + lag,
                            dur: event.dur.unwrap(),
                            amount: event.dose.unwrap(),
                            compartment: event.input.unwrap() - 1,
                        });
                    } else {
                        //dose
                        system.dose = Some(Dose {
                            time: event.time + lag,
                            amount: event.dose.unwrap(),
                            compartment: event.input.unwrap() - 1,
                        });
                    }
                } else if event.evid == 0 {
                    //obs
                    yout.push(y0[1] / params[2]);
                }
                if let Some(next_time) = scenario.times.get(index + 1) {
                    let mut stepper = Rk4::new(system.clone(), event.time, y0, *next_time, 0.1);
                    let _res = stepper.integrate();
                    let y = stepper.y_out();
                    y0 = *y.last().unwrap();
                    index += 1;
                }
            }
        }
        yout
    }
}

fn main() -> Result<()> {
    start(
        Engine::new(Ode {}),
        "examples/two_eq_lag/config.toml".to_string(),
        (0.1, 0.25, -0.001, 0.0),
    )?;
    Ok(())
}