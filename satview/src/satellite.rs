use sgp4::Prediction;
use chrono::NaiveDateTime;
use crate::satutil::{get_ecef, get_geodetic, get_teme};
use crate::coord_systems::{TEME, Geodetic, ECEF};

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

    pub fn update_sat_state(&mut self, sat_prediction: &Prediction, new_epoch: &NaiveDateTime) {
        let geo = get_geodetic(sat_prediction, new_epoch);

        self.geodetic_coordinates = geo;
        self.teme_coordinates = get_teme(&geo, &new_epoch);
        self.ecef_coordinates = get_ecef(&geo);
    }
}