use sgp4::Prediction;
use chrono::NaiveDateTime;
use crate::coordinate_systems::{TEME, Geodetic, ECEF};

/// A satellite should just be a place to store satellite information
/// coordinate Geodetic and TEME + sidereal
/// Update methods, etc...
pub struct Satellite { 
    pub geodetic_coordinates: Geodetic,
    pub teme_coordinates: TEME,
    pub ecef_coordinates: ECEF,
    pub sat_elements: sgp4::Elements,
}

impl Satellite {
    pub fn new(sat_elements: sgp4::Elements) -> Satellite {        
        Satellite { 
            geodetic_coordinates: Geodetic { 
                ..Default::default()
            },
            teme_coordinates: TEME { 
                ..Default::default()
            },
            ecef_coordinates: ECEF { 
                ..Default::default()
            },
            sat_elements: sat_elements,
        }
    }
}