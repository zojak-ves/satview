//! purpose: 
//!     this file defines the various coordinate system structs.
//! 

#[derive(Debug, Copy, Clone)]
pub struct Geodetic { 
    pub latitude: f64,
    pub longitude: f64,
    pub altitude: f64,
}
impl Default for Geodetic { 
    fn default() -> Geodetic { 
        Geodetic { 
            latitude: 0.0,
            longitude: 0.0,
            altitude: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct ECEF { 
    pub x: f64,
    pub y: f64,
    pub z: f64
}
impl Default for ECEF { 
    fn default() -> ECEF {
        ECEF { 
            x: 0.0,
            y: 0.0,
            z: 0.0,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct TEME { 
    pub pos_vector: [f64; 3],
    pub pos_magnitude: f64,
    pub velo_vector: [f64; 3],
    pub velo_magnitude: f64,
    pub sidereal: f64,
}
impl Default for TEME { 
    fn default() -> TEME {
        TEME { 
            pos_vector: [0.0, 0.0, 0.0],
            pos_magnitude: 0.0,
            velo_vector: [0.0, 0.0, 0.0],
            velo_magnitude: 0.0,
            sidereal: 0.0,
        }
    }
}