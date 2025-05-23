use crate::element::*;
use crate::Result;

use simba::scalar::SubsetOf;

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
    elements: Vec<crate::element::Element<T>>,
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
