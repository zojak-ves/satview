use chrono::NaiveDateTime;
use sgp4::iau_epoch_to_sidereal_time;
use sgp4::WGS84;
use crate::coord_systems::{ECEF, Geodetic, TEME};
use libm::fabs;

const FLAT_FACTOR: f64 = 1.0 / 298.26; // WGS84 flattening factor.
const E2: f64 = 6.69437999014e-3;      // Square of first eccentricity.
const OMEGA_E: f64 = 1.00273790934;
const SECONDS_PER_DAY: f64 = 86400.0;
const MFACTOR: f64 = (core::f64::consts::PI * 2.0) * (OMEGA_E / SECONDS_PER_DAY);

pub fn get_teme(geo_coords: &Geodetic, new_epoch: &NaiveDateTime) -> TEME { 
    let sidereal = to_sidereal(&new_epoch);

    let lat_rad = degrees_to_radians(&geo_coords.latitude);
    let lon_rad = degrees_to_radians(&geo_coords.longitude);

    // Calculate Local Mean Sidereal Time for observers longitude
    let theta = to_local_sidereal_time(lon_rad, &sidereal);

    let c: f64 = 1.0
        / (1.0 + FLAT_FACTOR * (FLAT_FACTOR - 2.0) * lat_rad.sin().powf(2.0)).sqrt();
    let s = (1.0 - FLAT_FACTOR).powf(2.0) * c;
    let achcp: f64 = (WGS84.ae * c + geo_coords.altitude) * lat_rad.cos();

    // X position in km
    // Y position in km
    // Z position in km
    // W magnitude in km
    let pos_x = achcp * theta.cos();
    let pos_y = achcp * theta.sin();
    let pos_z = (WGS84.ae * s + geo_coords.altitude) * lat_rad.sin();
    let pos_w = (pos_x * pos_x + pos_y * pos_y + pos_z * pos_z).sqrt();

    // X velocity in km/s
    // Y velocity in km/s
    // Z velocity in km/s
    // W magnitude in km/s
    let velo_x = -MFACTOR * pos_y;
    let velo_y = MFACTOR * pos_x;
    let velo_z = 0.0;
    let velo_w = (velo_x * velo_x + velo_y * velo_y + velo_z * velo_z).sqrt();

    TEME { 
        pos_vector: [pos_x, pos_y, pos_z],
        pos_magnitude: pos_w,
        velo_vector: [velo_x, velo_y, velo_z],
        velo_magnitude: velo_w,
        sidereal: sidereal,
    }
}

pub fn get_geodetic(propagation: &sgp4::Prediction, updated_epoch: &NaiveDateTime) -> Geodetic {
    let sidereal = to_sidereal(updated_epoch);

    let theta = propagation.position[1].atan2(propagation.position[0]);
    let r = ((propagation.position[0] * propagation.position[0]) + (propagation.position[1] * propagation.position[1])).sqrt();
    let e2 = FLAT_FACTOR * (2.0 - FLAT_FACTOR);
    let mut c: f64;
    let mut phi: f64;
    let mut cnt: i32 = 0;
    let mut lon: f64 = neg_pos_pi(theta - sidereal);
    let mut lat = propagation.position[2].atan2(r);
    
    loop { 
        phi = lat;
        c = 1.0 / (1.0 - e2 * phi.sin() * phi.sin()).sqrt();
        lat = (propagation.position[2] + WGS84.ae * c * e2 * phi.sin()).atan2(r);

        if fabs((lat - phi)) < 1e-10 || cnt >= 10 { 
            break;
        }
        cnt += 1;
    }

    let alt = r / lat.cos() - WGS84.ae * c;
    lat = radians_to_degrees(&lat);
    lon = radians_to_degrees(&lon);
    

    Geodetic { 
        latitude: lat,
        longitude: lon,
        altitude: alt,
    }
}

pub fn get_ecef(geodetic_coords: &Geodetic) -> ECEF { 
    let radians_lat = degrees_to_radians(&geodetic_coords.latitude);
    let radians_lon = degrees_to_radians(&geodetic_coords.longitude);
    let alt = geodetic_coords.altitude;
    let N = WGS84.ae / (1.0 - E2 * radians_lat.sin().powf(2.0)).sqrt(); // Prime vertical radius of curvature
    
    let ecef_x: f64 = (N + alt) * radians_lat.cos() * radians_lon.cos();
    let ecef_y: f64 = (N + alt) * radians_lat.cos() * radians_lon.sin();
    let ecef_z = ((1.0 - E2) * N + alt) * radians_lat.sin();

    ECEF { 
        x: ecef_x,
        y: ecef_y,
        z: ecef_z,
    }
}

fn neg_pos_pi(a: f64) -> f64 { 
    float_mod(a + core::f64::consts::PI, 2.0 * core::f64::consts::PI) - core::f64::consts::PI
}

fn float_mod(x: f64, y: f64) -> f64 { 
    if y == 0.0 { 
        return x;
    }
    x - y * (x / y).floor()
}

pub fn radians_to_degrees(radians: &f64) -> f64 { 
    radians * 180.0 / core::f64::consts::PI
}

pub fn degrees_to_radians(degrees: &f64) -> f64 { 
    degrees * core::f64::consts::PI / 180.0
}

fn wrap_two_pi(val: f64) -> f64 { 
    float_mod(val, core::f64::consts::PI * 2.0)
}

pub fn to_local_sidereal_time(longitude: f64, sidereal_time: &f64) -> f64 { 
    wrap_two_pi(sidereal_time + longitude)
}

pub fn to_sidereal(date_time: &NaiveDateTime) -> f64 { 
    iau_epoch_to_sidereal_time(sgp4::julian_years_since_j2000(date_time))
}