use crate::algorithms::initialize_algorithm;
use crate::prelude::{
    output::NPResult,
    predict::{Engine, Predict},
    *,
};
use crate::routines::datafile::Scenario;
use crate::routines::settings::*;

use csv::{ReaderBuilder, WriterBuilder};
use eyre::Result;

use ndarray::Array2;
use ndarray_csv::Array2Reader;
use predict::sim_obs;
use std::fs::File;
use std::thread::spawn;
use std::time::Instant;
use tokio::sync::mpsc::{self};

/// Simulate predictions from a model and prior distribution
///
/// This function is used to simulate predictions from a model and prior distribution.
/// The output is a CSV file with the following columns:
/// - `id`: subject ID, corresponding to the desired dose regimen
/// - `point`: support point index (0-indexed)
/// - `time`: prediction time
/// - `pred`: simulated prediction
///
/// # Arguments
/// The user can specify the desired settings in a TOML configuration file, see `routines::settings::simulator` for details.
/// - `idelta`: the interval between predictions. Default is 0.0.
/// - `tad`: the time after dose, which if greater than the last prediction time is the time for which it will predict . Default is 0.0.
pub fn simulate<S>(engine: Engine<S>, settings_path: String) -> Result<()>
where
    S: Predict<'static> + std::marker::Sync + std::marker::Send + 'static + Clone,
{
    let settings: Settings = read_settings(settings_path).unwrap();
    let theta_file = File::open(settings.paths.prior.unwrap()).unwrap();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_reader(theta_file);
    let theta: Array2<f64> = reader.deserialize_array2_dynamic().unwrap();

    // Expand data
    let idelta = settings.config.idelta;
    let tad = settings.config.tad;
    let mut scenarios = datafile::parse(&settings.paths.data).unwrap();
    scenarios.iter_mut().for_each(|scenario| {
        *scenario = scenario.add_event_interval(idelta, tad);
    });

    // Perform simulation
    let ypred = sim_obs(&engine, &scenarios, &theta, false);

    // Prepare writer
    let sim_file = File::create("simulation_output.csv").unwrap();
    let mut sim_writer = WriterBuilder::new()
        .has_headers(false)
        .from_writer(sim_file);
    sim_writer
        .write_record(["id", "point", "time", "pred"])
        .unwrap();

    // Write output
    for (id, scenario) in scenarios.iter().enumerate() {
        let time = scenario.obs_times.clone();
        for (point, _spp) in theta.rows().into_iter().enumerate() {
            for (i, time) in time.iter().enumerate() {
                sim_writer.write_record(&[
                    id.to_string(),
                    point.to_string(),
                    time.to_string(),
                    ypred.get((id, point)).unwrap().get(i).unwrap().to_string(),
                ])?;
            }
        }
    }
    Ok(())
}

/// Primary entrypoint for NPcore
///
/// This function is the primary entrypoint for NPcore, and is used to run the algorithm.
/// The settings for this function is specified in a TOML configuration file, see `routines::settings::run` for details.
pub fn start<S>(engine: Engine<S>, settings_path: String) -> Result<NPResult>
where
    S: Predict<'static> + std::marker::Sync + std::marker::Send + 'static + Clone,
{
    let now = Instant::now();
    let settings = match read_settings(settings_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error reading settings: {:?}", e);
            std::process::exit(-1);
        }
    };
    let (tx, rx) = mpsc::unbounded_channel::<Comm>();
    let maintx = tx.clone();
    logger::setup_log(&settings, tx.clone());
    tracing::info!("Starting NPcore");

    // Read input data and remove excluded scenarios (if any)
    let mut scenarios = datafile::parse(&settings.paths.data).unwrap();
    if let Some(exclude) = &settings.config.exclude {
        for val in exclude {
            scenarios.remove(val.as_ptr() as usize);
        }
    }

    // Provide information of the input data
    tracing::info!(
        "Datafile contains {} subjects with a total of {} observations",
        scenarios.len(),
        scenarios.iter().map(|s| s.obs_times.len()).sum::<usize>()
    );

    // Spawn new thread for TUI
    let settings_tui = settings.clone();
    let handle = if settings.config.tui {
        spawn(move || {
            start_ui(rx, settings_tui).expect("Failed to start TUI");
        })
    } else {
        // Drop messages if TUI is not enabled to reduce memory usage
        spawn(move || {
            drop_messages(rx);
        })
    };

    // Initialize algorithm and run
    let mut algorithm = initialize_algorithm(engine.clone(), settings.clone(), scenarios, tx);
    let result = algorithm.fit();
    tracing::info!("Total time: {:.2?}", now.elapsed());

    // Write output files (if configured)
    if settings.config.output {
        let idelta = settings.config.idelta;
        let tad = settings.config.tad;
        result.write_outputs(true, &engine, idelta, tad);
    }

    tracing::info!("Program complete");
    maintx.send(Comm::StopUI).unwrap();
    handle.join().unwrap();

    Ok(result)
}

/// Alternative entrypoint, primarily meant for third-party libraries or APIs
///
/// This entrypoint takes an `Engine` (from the model), `Data` from the settings, and `scenarios` containing dose information and observations
///
/// It does not write any output files, and does not start a TUI.
///
/// Returns an NPresult object
pub fn start_internal<S>(
    engine: Engine<S>,
    settings: Settings,
    scenarios: Vec<Scenario>,
) -> Result<NPResult>
where
    S: Predict<'static> + std::marker::Sync + std::marker::Send + 'static + Clone,
{
    let now = Instant::now();
    let (tx, rx) = mpsc::unbounded_channel::<Comm>();
    logger::setup_log(&settings, tx.clone());

    let mut algorithm = initialize_algorithm(engine.clone(), settings.clone(), scenarios, tx);

    let _ = spawn(move || {
        drop_messages(rx);
    });

    let result = algorithm.fit();
    tracing::info!("Total time: {:.2?}", now.elapsed());
    Ok(result)
}

fn drop_messages(mut rx: mpsc::UnboundedReceiver<Comm>) {
    loop {
        match rx.try_recv() {
            Ok(comm) => match comm {
                Comm::StopUI => break,
                _ => {}
            },
            Err(_e) => {}
        }
    }
}
