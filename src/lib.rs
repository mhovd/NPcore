//! NPcore is a framework for developing and running non-parametric algorithms for population pharmacokinetic modelling

pub mod algorithms;
pub mod routines {
    pub mod datafile;
    pub mod initialization;
    pub mod optimization {
        pub mod d_optimizer;
        pub mod optim;
    }
    pub mod output;
    pub mod condensation {
        pub mod prune;
    }
    pub mod expansion {
        pub mod adaptative_grid;
    }

    pub mod settings;
    pub mod evaluation {

        pub mod ipm;
        pub mod prob;
        pub mod qr;
        pub mod sigma;
    }
    pub mod simulation {
        pub mod predict;
    }
}
pub mod entrypoints;
pub mod logger;
pub mod tui;

pub mod prelude {
    pub use crate::algorithms;
    pub use crate::entrypoints::simulate;
    pub use crate::entrypoints::start;
    pub use crate::entrypoints::start_internal;
    pub use crate::logger;
    pub use crate::prelude::evaluation::{prob, sigma, *};
    pub use crate::routines::condensation;
    pub use crate::routines::expansion::*;
    pub use crate::routines::initialization::*;
    pub use crate::routines::optimization;
    pub use crate::routines::simulation::*;
    pub use crate::routines::*;
    pub use crate::tui::ui::*;
}

//Tests
mod tests;
