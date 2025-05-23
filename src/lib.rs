//! > vizz a vi
//!
//! # Red Circle
//! ```
//! use vizzavi::backend::bitmap::*;
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

pub mod backend;
pub mod element;
pub mod figure;

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
