use vizzavi::backend::bitmap::*;

fn main() -> Result<()> {
    let mut figure: Figure<f32> = Figure::new();
    figure.set_xlim([0., 300.]);
    figure.set_ylim([0., 300.]);

    figure.color_by_func(|x| Some(((x[0] - 100.) / 100.).powf(2.0)), CmapGreys);

    let rectangle = Rectangle {
        color: [250, 250, 250],
        min: [0f32; 2].into(),
        max: [100f32; 2].into(),
    };
    figure.add_element(rectangle);

    let circle = Circle {
        middle: [150., 200.].into(),
        radius: 33.,
        color: [200, 0, 0],
    };
    figure.add_element(circle);

    figure.save_image([800, 800], "temp.png")?;

    Ok(())
}
