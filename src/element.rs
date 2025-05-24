#[derive(Clone, Debug)]
pub struct Rectangle<T> {
    pub min: nalgebra::Vector2<T>,
    pub max: nalgebra::Vector2<T>,
    pub color: Color,
}

pub type Color = [u8; 3];

#[derive(Clone, Debug)]
pub struct Circle<T> {
    pub middle: nalgebra::Vector2<T>,
    pub radius: T,
    pub color: Color,
}

#[derive(Clone, Debug)]
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

#[derive(Clone, Debug)]
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
