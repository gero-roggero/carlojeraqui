//! Grabación: captura de pantalla y audio, codificación en la GPU, buffer en RAM y guardado de clips.

pub mod audio;
pub mod capture;
pub mod clock;
mod convert;
mod encoder;
mod game;
mod mux;
pub mod recorder;
