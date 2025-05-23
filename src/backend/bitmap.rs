pub use crate::element::*;
pub use crate::figure::*;
pub use crate::*;

use simba::scalar::SubsetOf;

impl<T> Element<T> {
    pub(crate) fn draw_to_axis(
        &self,
        axis: &Figure<T>,
        canvas: &mut ndarray::Array3<u8>,
    ) -> Result<()>
    where
        T: nalgebra::RealField + num::cast::AsPrimitive<usize>,
        usize: simba::scalar::SubsetOf<T>,
    {
        let size_pixels = canvas.dim();
        let size_pixels = nalgebra::Vector2::from([size_pixels.0, size_pixels.1]);

        match self {
            Element::Triangle(Triangle { points, color }) => {
                todo!()
            }
            Element::Rectangle(Rectangle { color, min, max }) => {
                let color = ndarray::Array1::from_iter(*color);
                let pix_min = axis.coordinate_to_pixel(min, size_pixels);
                let pix_max = axis.coordinate_to_pixel(max, size_pixels);
                canvas
                    .slice_mut(ndarray::s![
                        pix_min[0]..pix_max[0],
                        pix_min[1]..pix_max[1],
                        ..
                    ])
                    .assign(&color);
            }
            Element::Circle(Circle {
                middle,
                radius,
                color,
            }) => {
                let color = ndarray::Array1::from_iter(*color);
                let min = middle.add_scalar(-*radius);
                let max = middle.add_scalar(*radius);
                let pix_min = axis.coordinate_to_pixel(&min, size_pixels);
                let pix_max = axis.coordinate_to_pixel(&max, size_pixels);

                let two = T::one() + T::one();
                let pix_middle: nalgebra::Vector2<T> =
                    (pix_max + pix_min).map(|x| x.to_superset() / two);
                for m in pix_min[0]..pix_max[0] {
                    let t: T = (m.to_superset() - pix_middle[0]).abs()
                        / (pix_max[0].to_superset() - pix_middle[0]);
                    let s = (T::one() - t.powf(two)).sqrt();
                    let q = s * (pix_max[1] - pix_min[1]).to_superset() / two;

                    let pix_low = (pix_middle[1] - q).as_();
                    let pix_high = (pix_middle[1] + q).as_();
                    canvas
                        .slice_mut(ndarray::s![m, pix_low..pix_high, ..])
                        .assign(&color);
                }
            }
            Element::FuncEval(func, cmap) => {
                let shape = canvas.dim();
                for i in 0..shape.0 {
                    for j in 0..shape.1 {
                        let pos = axis.pixel_to_coordinate(&[i, j], size_pixels);
                        let value = func(pos);
                        if let Some(v) = value {
                            let color = cmap.get_color(v);
                            canvas[(i, j, 0)] = color[0];
                            canvas[(i, j, 1)] = color[1];
                            canvas[(i, j, 2)] = color[2];
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
