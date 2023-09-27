extern crate flyer;
use flyer::Aircraft;

use plotters::prelude::*;

use aerso::types::*;

fn plot_line(x_data: Vec<f64>, y_data: Vec<f64>) -> Result<(), Box<dyn std::error::Error>> {
    // Create a drawing area and specify the output file format
    let root = BitMapBackend::new("plot.png", (800, 600)).into_drawing_area();
    root.fill(&WHITE)?;

    // Create a chart context
    let mut chart = ChartBuilder::on(&root)
        .caption("Line Plot", ("sans-serif", 30))
        .x_label_area_size(40)
        .y_label_area_size(40)
        .build_cartesian_2d(0.0..1000.0, 80.0..120.0)?;

    // Configure the axes
    chart
        .configure_mesh()
        .x_desc("X Axis")
        .y_desc("Y Axis")
        .draw()?;

    // Plot the data as a line
    chart
        .draw_series(LineSeries::new(
            x_data.iter().zip(y_data.iter()).map(|(x, y)| (*x, *y)),
            &RED,
        ))?;

    Ok(())
}


fn simulate() {

    const FPS: u32 = 100;
    const EXP_LEN: f32 = 1000.0;

    let dt = 1.0/FPS as f64;
    let airspeed = 100.0;
    let alt = 1000.0;
    
    //
    // -0.31249638146843983, 0.008771154220320694, 0.5695319922972387
    let trim_cond = vec![-0.3490658503988659, 0.009204865735487735, 0.5053002990343323];

    let mut aircraft = Aircraft::new(
        "TO",
        Vector3::new(0.0, 0.0, alt),
        Vector3::new(airspeed, 0.0, 0.0),
        UnitQuaternion::from_euler_angles(0.0, trim_cond[0], 0.0),
        Vector3::zeros()
    );

    let controls = vec![trim_cond[1], 0.0, trim_cond[2], 0.0];
    let mut time = 0.0;

    let mut times: Vec<f64> = Vec::new();
    let mut us: Vec<f64> = Vec::new();

    for _ in 0..(FPS * (EXP_LEN as u32)) {
        aircraft.aff_body.step(dt, &controls);
        time += dt;

        times.push(time);

        us.push(aircraft.velocity()[0])
    }

    // Plot a chart
    let _ = plot_line(times, us);
}

fn main() {
    simulate();
}