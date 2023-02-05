#[cfg(test)]
use super::base::*;

#[test]
fn basic_sobol(){
    assert_eq!(samplers::sobol(5, vec![(0.,1.),(0.,1.),(0.,1.)], 347), ndarray::array![
        [0.10731888, 0.14647412, 0.58510387],
        [0.9840305, 0.76333654, 0.19097507],
        [0.3847711, 0.73466134, 0.2616291],
        [0.70233, 0.41038263, 0.9158684],
        [0.60167587, 0.61712956, 0.62639713]
    ])
}

#[test]
fn scaled_sobol(){
    assert_eq!(samplers::sobol(5, vec![(0.,1.),(0.,2.),(-1.,1.)], 347), ndarray::array![
        [0.10731888, 0.29294825, 0.17020774],
        [0.9840305, 1.5266731, -0.61804986],
        [0.3847711, 1.4693227, -0.4767418],
        [0.70233, 0.82076526, 0.8317368],
        [0.60167587, 1.2342591, 0.25279427]
    ])
}

#[test]
fn read_mandatory_settings(){
    let settings = settings::read();
    assert_eq!(settings.paths.data, "data.csv");
    assert_eq!(settings.config.cycles, 1024);
    assert_eq!(settings.config.engine, "NPAG");
}