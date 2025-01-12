//! purpose: 
//!     observer.rs defines an observer.
//!     What is an observer? For now an observer is any terrestrial body
//!     which is "observing" an orbiting body. In our case these orbiting bodies are satellites
//!     and the terrestrial bodies are "gateways" and "terminals". 
//!     For purposes of this code the differences between gateways and terminals are not
//!     important, just know that they are different.

use sgp4::Geopotential;
use crate::coordinate_systems::Geodetic;

/// An observer has:
/// geodetic_coordinates
/// look_angle to satellite
/// path_loss in dB

#[derive(Copy, Clone)]
pub struct Observer { 
    pub geodetic_coordinates: Geodetic,
    pub look_angle: f64,
    pub path_loss: f64,
}

impl Observer {
    pub fn new() -> Observer { 
        Observer {
            geodetic_coordinates: Geodetic { 
                ..Default::default()
            },
            look_angle: 0.0,
            path_loss : 0.0,
        }
    }
}