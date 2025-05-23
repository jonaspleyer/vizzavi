//! > vizz a vi
//!
//!
//! # Red Circle
//! ```
//! use vizzavi::*;
//!
//! let mut figure = Figure::new();
//! figure.set_xlim([0., 400.]);
//! figure.set_ylim([0., 300.]);
//!
//! let circle = Circle {
//!     middle: [150., 200.].into(),
//!     radius: 30.,
//!     color: [200, 0, 0],
//! };
//! figure.add_element(circle);
//!
//! figure.save_image([400, 300], "figures/red-circle.png")?;
//! # Ok::<(), ErrorTypes>(())
//! ```
//!
//! <img src="https://raw.githubusercontent.com/jonaspleyer/vizz/refs/heads/main/figures/red-circle.png">

// #![warn(missing_docs)]

use simba::scalar::SubsetOf;

#[derive(Debug, thiserror::Error)]
pub enum ErrorTypes {
    #[error("{0}")]
    ImageError(#[from] image::error::ImageError),
    #[error("{0}")]
    ImageCreation(&'static str),
    #[error("{0}")]
    CastError(&'static str),
}

pub type Result<T> = core::result::Result<T, ErrorTypes>;

pub struct Rectangle<T> {
    pub min: nalgebra::Vector2<T>,
    pub max: nalgebra::Vector2<T>,
    pub color: Color,
}

pub type Color = [u8; 3];

pub struct Circle<T> {
    pub middle: nalgebra::Vector2<T>,
    pub radius: T,
    pub color: Color,
}

pub struct Triangle<T> {
    pub points: nalgebra::Matrix3x2<T>,
    pub color: Color,
}

pub trait ColorMap<T>
where
    T: num::Zero + num::One,
{
    fn get_color_normalized(&self, h: T, min: T, max: T) -> [u8; 3];
    fn get_color(&self, h: T) -> [u8; 3] {
        self.get_color_normalized(h, T::zero(), T::one())
    }
}

pub struct CmapGreys;

impl<T> ColorMap<T> for CmapGreys
where
    T: nalgebra::RealField + num::cast::AsPrimitive<u8>,
    u8: simba::scalar::SubsetOf<T>,
{
    fn get_color_normalized(&self, h: T, min: T, max: T) -> [u8; 3] {
        use simba::scalar::SubsetOf;
        let t = (h - min) / (max - min).clamp(T::zero(), T::one());
        let c = (t * u8::MAX.to_superset()).as_();
        [c; 3]
    }
}

pub enum Element<T> {
    Rectangle(Rectangle<T>),
    Triangle(Triangle<T>),
    Circle(Circle<T>),
    FuncEval(
        Box<dyn Fn(nalgebra::Vector2<T>) -> Option<T>>,
        Box<dyn ColorMap<T>>,
    ),
}

impl<T> Element<T> {
    fn draw_to_axis(&self, axis: &Figure<T>, canvas: &mut ndarray::Array3<u8>) -> Result<()>
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

impl<T> From<Triangle<T>> for Element<T> {
    fn from(value: Triangle<T>) -> Self {
        Self::Triangle(value)
    }
}

impl<T> From<Rectangle<T>> for Element<T> {
    fn from(value: Rectangle<T>) -> Self {
        Self::Rectangle(value)
    }
}

impl<T> From<Circle<T>> for Element<T> {
    fn from(value: Circle<T>) -> Self {
        Self::Circle(value)
    }
}

pub struct Origin {
    is_left: bool,
    is_bottom: bool,
}

impl Default for Origin {
    fn default() -> Self {
        Origin {
            is_left: true,
            is_bottom: true,
        }
    }
}

impl Origin {
    pub fn switchx(&mut self) {
        self.is_left = !self.is_left;
    }

    pub fn switchy(&mut self) {
        self.is_bottom = !self.is_bottom;
    }
}

pub struct Figure<T> {
    corners_units: nalgebra::Matrix2<T>,
    elements: Vec<Element<T>>,
    origin: Origin,
}

impl<T> Default for Figure<T>
where
    T: nalgebra::RealField + num::cast::AsPrimitive<usize>,
    usize: simba::scalar::SubsetOf<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Figure<T>
where
    T: nalgebra::RealField + num::cast::AsPrimitive<usize>,
    usize: simba::scalar::SubsetOf<T>,
{
    pub fn new() -> Self {
        Self {
            corners_units: nalgebra::matrix![T::zero(), T::zero(); T::one(), T::one()],
            elements: Vec::new(),
            origin: Origin {
                is_left: true,
                is_bottom: true,
            },
        }
    }

    pub fn add_element(&mut self, element: impl Into<Element<T>>) {
        self.elements.push(element.into());
    }

    pub fn color_by_func(
        &mut self,
        func: impl 'static + Fn(nalgebra::Vector2<T>) -> Option<T>,
        cmap: impl 'static + ColorMap<T>,
    ) {
        self.add_element(Element::FuncEval(Box::new(func), Box::new(cmap)));
    }

    pub fn save_image<I>(&self, size: I, path: impl Into<std::path::PathBuf>) -> Result<()>
    where
        I: Into<[usize; 2]>,
    {
        let size_pixels: [usize; 2] = size.into();
        let size_pixels = nalgebra::Vector2::from(size_pixels);

        let mut canvas = ndarray::Array3::<u8>::zeros((size_pixels[0], size_pixels[1], 3));

        for element in self.elements.iter() {
            element.draw_to_axis(self, &mut canvas)?;
        }

        let Origin { is_left, is_bottom } = self.origin;
        if !is_left {
            canvas.invert_axis(ndarray::Axis(0));
        }
        if is_bottom {
            canvas.invert_axis(ndarray::Axis(1));
        }

        let img = image::RgbImage::from_fn(size_pixels[0] as u32, size_pixels[1] as u32, |i, j| {
            image::Rgb([
                canvas[(i as usize, j as usize, 0)],
                canvas[(i as usize, j as usize, 1)],
                canvas[(i as usize, j as usize, 2)],
            ])
        });

        let path: std::path::PathBuf = path.into();
        let img_format = image::ImageFormat::from_path(&path)?;
        img.save_with_format(path, img_format)?;
        Ok(())
    }

    pub fn get_dx(&self) -> nalgebra::Vector2<T> {
        (self.corners_units.row(1) - self.corners_units.row(0)).transpose()
    }

    pub fn coordinate_to_pixel(
        &self,
        pos: &nalgebra::Vector2<T>,
        size_pixels: nalgebra::Vector2<usize>,
    ) -> nalgebra::Vector2<usize> {
        use simba::scalar::SubsetOf;
        let dx = self.get_dx();
        (pos - self.corners_units.row(0).transpose())
            .component_div(&dx)
            .zip_map(&size_pixels, |a, b| (a * b.to_superset()).round().as_())
    }

    pub fn pixel_to_coordinate(
        &self,
        pix: &[usize; 2],
        size_pixels: nalgebra::Vector2<usize>,
    ) -> nalgebra::Vector2<T> {
        let dx = self.get_dx().cast();
        let pix = nalgebra::Vector2::<T>::from([pix[0].to_superset(), pix[1].to_superset()]);
        let dx_pix = size_pixels.cast();
        pix.component_div(&dx_pix).component_mul(&dx)
    }

    pub fn set_xlim(&mut self, lims: impl Into<nalgebra::Vector2<T>>) {
        let lims: nalgebra::Vector2<T> = lims.into();
        self.corners_units.set_column(0, &lims);
    }

    pub fn set_ylim(&mut self, lims: impl Into<nalgebra::Vector2<T>>) {
        let lims: nalgebra::Vector2<T> = lims.into();
        self.corners_units.set_column(1, &lims);
    }
}
