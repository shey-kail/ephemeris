// JPL 历表支持模块（精准模式）
//
// 使用 anise 库加载和计算 JPL DE 系列历表（DE431/DE441）
// 提供高精度的太阳、月球、行星位置计算

use anise::prelude::*;
use anise::constants::frames::{EARTH_J2000, SUN_J2000, MOON_J2000};
use crate::internal::planet::Planet;
use std::path::Path;
use std::sync::Arc;

/// JPL 历表类型
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JplEphemerisType {
    DE431,
    DE441,
    DE441Lite,
}

impl JplEphemerisType {
    pub fn file_paths(&self) -> Vec<String> {
        let workspace_root = env!("CARGO_MANIFEST_DIR").to_string() + "/..";
        match self {
            JplEphemerisType::DE431 => vec![
                workspace_root.clone() + "/bsp/431/de431_part-1.bsp",
                workspace_root.clone() + "/bsp/431/de431_part-2.bsp",
            ],
            JplEphemerisType::DE441 => vec![
                workspace_root.clone() + "/bsp/441/de441_part-1.bsp",
                workspace_root.clone() + "/bsp/441/de441_part-2.bsp",
            ],
            JplEphemerisType::DE441Lite => vec![
                workspace_root + "/bsp/441/de441_3000bc_3000ad.bsp",
            ],
        }
    }
    pub fn year_range(&self) -> (i32, i32) {
        match self {
            JplEphemerisType::DE431 => (-13000, 17000),
            JplEphemerisType::DE441 => (-13000, 17000),
            JplEphemerisType::DE441Lite => (-3000, 3000),
        }
    }
}

#[derive(Clone)]
pub struct JplEphemeris {
    almanac: Arc<Almanac>,
    ephemeris_type: JplEphemerisType,
}

impl JplEphemeris {
    pub fn new(ephemeris_type: JplEphemerisType) -> Result<Self, String> {
        let mut almanac = Almanac::default();
        for path in ephemeris_type.file_paths() {
            if !Path::new(&path).exists() {
                return Err(format!("历表文件不存在：{}", path));
            }
            let spk = SPK::load(&path).map_err(|e| format!("加载失败：{}", e))?;
            almanac = almanac.with_spk(spk);
        }
        Ok(Self { almanac: Arc::new(almanac), ephemeris_type })
    }

    fn compute_raw_position(&self, target: Frame, jd_tdb: f64, use_aberration: bool) -> Result<((f64, f64, f64), f64, anise::math::Vector3), String> {
        let epoch = Epoch::from_jde_tdb(jd_tdb);
        // 在 anise 0.9.6 中，Aberration::CN_S 等常量已经是 Option<Aberration> 类型
        let ab = if use_aberration { Aberration::CN_S } else { Aberration::NONE };
        
        let state = self.almanac
            .translate(target, EARTH_J2000, epoch, ab)
            .map_err(|e| format!("anise translate 失败：{}", e))?;
        
        let pos = (state.radius_km.x, state.radius_km.y, state.radius_km.z);
        let r_km = state.radius_km.norm();
        Ok((pos, r_km, state.velocity_km_s))
    }

    fn transform_to_apparent_ecliptic(&self, pos_j2000: (f64, f64, f64), jd_tdb: f64, apply_nutation: bool) -> (f64, f64) {
        let t = (jd_tdb - 2451545.0) / 36525.0;
        use crate::internal::corrections::{apply_precession_iau2006, equatorial_to_ecliptic};
        use crate::internal::ephemeris::obliquity;
        use crate::internal::math_utils::xyz2llr;

        // 使用 IAU 2006 岁差模型
        let pos_mean_equ = apply_precession_iau2006(pos_j2000, t);
        let eps_mean = obliquity(t);
        let pos_mean_ecl = equatorial_to_ecliptic(pos_mean_equ, eps_mean);
        let (lon_mean, lat_mean, _) = xyz2llr(pos_mean_ecl);

        let lon_final = if apply_nutation {
            let (d_psi, _) = crate::internal::nutation_iau2000a_impl::nutation_iau2000a(jd_tdb);
            lon_mean + d_psi
        } else {
            lon_mean
        };
        (lon_final, lat_mean)
    }

    pub fn solar_position(&self, jd_tdb: f64) -> Result<SolarPosition, String> {
        let (pos_j2000, r_km, vel) = self.compute_raw_position(SUN_J2000, jd_tdb, true)?;
        let (lon, lat) = self.transform_to_apparent_ecliptic(pos_j2000, jd_tdb, true);
        Ok(SolarPosition {
            epoch: jd_tdb,
            longitude: crate::internal::math_utils::rad2mrad(lon),
            latitude: lat,
            distance: r_km / crate::internal::constants::CS_AU,
            velocity_x: vel.x, velocity_y: vel.y, velocity_z: vel.z,
        })
    }

    pub fn lunar_position(&self, jd_tdb: f64) -> Result<LunarPosition, String> {
        let (pos_j2000, r_km, vel) = self.compute_raw_position(MOON_J2000, jd_tdb, true)?;
        let (lon, lat) = self.transform_to_apparent_ecliptic(pos_j2000, jd_tdb, true);
        Ok(LunarPosition {
            epoch: jd_tdb,
            longitude: crate::internal::math_utils::rad2mrad(lon),
            latitude: lat,
            distance: r_km / crate::internal::constants::CS_AU,
            velocity_x: vel.x, velocity_y: vel.y, velocity_z: vel.z,
        })
    }

    pub fn planet_position(&self, planet: Planet, jd_tdb: f64) -> Result<PlanetPosition, String> {
        let frame = self.get_planet_frame(planet);
        let (pos_j2000, r_km, vel) = self.compute_raw_position(frame, jd_tdb, true)?;
        let (lon, lat) = self.transform_to_apparent_ecliptic(pos_j2000, jd_tdb, true);
        Ok(PlanetPosition {
            planet, epoch: jd_tdb,
            longitude: crate::internal::math_utils::rad2mrad(lon),
            latitude: lat,
            distance: r_km / crate::internal::constants::CS_AU,
            velocity_x: vel.x, velocity_y: vel.y, velocity_z: vel.z,
        })
    }

    pub fn planet_apparent_position(
        &self,
        planet: Planet,
        jd_tdb: f64,
        apply_aberration: bool,
        apply_nutation: bool,
    ) -> Result<ApparentPlanetPosition, String> {
        let frame = self.get_planet_frame(planet);
        let (pos_geo_j2000, _, _) = self.compute_raw_position(frame, jd_tdb, false)?;
        let (lon_geo, lat_geo) = self.transform_to_apparent_ecliptic(pos_geo_j2000, jd_tdb, false);

        let (pos_app_j2000, r_km_app, _) = self.compute_raw_position(frame, jd_tdb, apply_aberration)?;
        let (lon_app, lat_app) = self.transform_to_apparent_ecliptic(pos_app_j2000, jd_tdb, apply_nutation);

        Ok(ApparentPlanetPosition {
            planet, epoch: jd_tdb,
            geometric_longitude: crate::internal::math_utils::rad2mrad(lon_geo),
            geometric_latitude: lat_geo,
            apparent_longitude: crate::internal::math_utils::rad2mrad(lon_app),
            apparent_latitude: lat_app,
            distance: r_km_app / crate::internal::constants::CS_AU,
        })
    }

    fn get_planet_frame(&self, planet: Planet) -> Frame {
        match planet {
            Planet::Mercury => anise::constants::frames::MERCURY_J2000,
            Planet::Venus => anise::constants::frames::VENUS_J2000,
            Planet::Moon => anise::constants::frames::MOON_J2000,
            Planet::Sun => anise::constants::frames::SUN_J2000,
            Planet::Earth => EARTH_J2000,
            Planet::Mars => Frame::from_ephem_j2000(4),
            Planet::Jupiter => Frame::from_ephem_j2000(5),
            Planet::Saturn => Frame::from_ephem_j2000(6),
            Planet::Uranus => Frame::from_ephem_j2000(7),
            Planet::Neptune => Frame::from_ephem_j2000(8),
            Planet::Pluto => Frame::from_ephem_j2000(9),
        }
    }

    pub fn find_new_moon(&self, start_jd: f64, max_days: Option<f64>) -> Result<f64, String> {
        let max_days = max_days.unwrap_or(30.0);
        let search_func = |jd: f64| -> Result<f64, String> {
            let sun = self.solar_position(jd)?;
            let moon = self.lunar_position(jd)?;
            let mut diff = moon.longitude - sun.longitude;
            while diff > std::f64::consts::PI { diff -= 2.0 * std::f64::consts::PI; }
            while diff < -std::f64::consts::PI { diff += 2.0 * std::f64::consts::PI; }
            Ok(diff)
        };
        self.brent_search(start_jd, max_days, &search_func, 0.0)
    }
    
    pub fn find_full_moon(&self, start_jd: f64, max_days: Option<f64>) -> Result<f64, String> {
        let max_days = max_days.unwrap_or(30.0);
        let search_func = |jd: f64| -> Result<f64, String> {
            let sun = self.solar_position(jd)?;
            let moon = self.lunar_position(jd)?;
            let mut diff = moon.longitude - sun.longitude;
            while diff < 0.0 { diff += 2.0 * std::f64::consts::PI; }
            while diff > 2.0 * std::f64::consts::PI { diff -= 2.0 * std::f64::consts::PI; }
            Ok(diff - std::f64::consts::PI)
        };
        self.brent_search(start_jd, max_days, &search_func, 0.0)
    }
    
    pub fn find_solar_term(&self, start_jd: f64, term_index: usize) -> Result<f64, String> {
        let target_lon = (term_index * 15) as f64 * std::f64::consts::PI / 180.0;
        let search_func = |jd: f64| -> Result<f64, String> {
            let sun = self.solar_position(jd)?;
            let mut diff = sun.longitude - target_lon;
            while diff > std::f64::consts::PI { diff -= 2.0 * std::f64::consts::PI; }
            while diff < -std::f64::consts::PI { diff += 2.0 * std::f64::consts::PI; }
            Ok(diff)
        };
        self.brent_search(start_jd, 35.0, &search_func, 0.0)
    }
    
    fn brent_search(&self, start_jd: f64, max_days: f64, func: &dyn Fn(f64) -> Result<f64, String>, target: f64) -> Result<f64, String> {
        let step = 0.2; let mut jd = start_jd;
        let mut prev_val = func(jd)? - target;
        let iterations = (max_days / step) as i32 + 1;
        for _ in 0..iterations {
            jd += step;
            let curr_val = match func(jd) { Ok(v) => v - target, Err(_) => break };
            if prev_val * curr_val < 0.0 { return self.brent_refine(jd - step, jd, func, target); }
            prev_val = curr_val;
        }
        Err(format!("在 {} 天内未找到零点", max_days))
    }
    
    fn brent_refine(&self, jd_low: f64, jd_high: f64, func: &dyn Fn(f64) -> Result<f64, String>, target: f64) -> Result<f64, String> {
        let mut a = jd_low; let mut b = jd_high;
        let mut fa = func(a)? - target;
        for _ in 0..60 {
            let mid = (a + b) / 2.0;
            let fmid = func(mid)? - target;
            if fmid.abs() < 1e-12 || (b - a).abs() < 1e-12 { return Ok(mid); }
            if fa * fmid < 0.0 { b = mid; } else { a = mid; fa = fmid; }
        }
        Ok((a + b) / 2.0)
    }
    
    pub fn ephemeris_type(&self) -> JplEphemerisType { self.ephemeris_type }
    pub fn year_range(&self) -> (i32, i32) { self.ephemeris_type.year_range() }
}

#[derive(Debug, Clone)]
pub struct PlanetPosition { pub planet: Planet, pub epoch: f64, pub longitude: f64, pub latitude: f64, pub distance: f64, pub velocity_x: f64, pub velocity_y: f64, pub velocity_z: f64 }
impl PlanetPosition {
    pub fn longitude_deg(&self) -> f64 { self.longitude.to_degrees() }
    pub fn latitude_deg(&self) -> f64 { self.latitude.to_degrees() }
    pub fn distance_km(&self) -> f64 { self.distance * 149597870.7 }
}
#[derive(Debug, Clone)]
pub struct ApparentPlanetPosition { pub planet: Planet, pub epoch: f64, pub geometric_longitude: f64, pub geometric_latitude: f64, pub apparent_longitude: f64, pub apparent_latitude: f64, pub distance: f64 }
impl ApparentPlanetPosition {
    pub fn apparent_longitude_deg(&self) -> f64 { self.apparent_longitude.to_degrees() }
    pub fn apparent_latitude_deg(&self) -> f64 { self.apparent_latitude.to_degrees() }
    pub fn geometric_longitude_deg(&self) -> f64 { self.geometric_longitude.to_degrees() }
}
#[derive(Debug, Clone)]
pub struct SolarPosition { pub epoch: f64, pub longitude: f64, pub latitude: f64, pub distance: f64, pub velocity_x: f64, pub velocity_y: f64, pub velocity_z: f64 }
impl SolarPosition {
    pub fn longitude_deg(&self) -> f64 { self.longitude.to_degrees() }
    pub fn latitude_deg(&self) -> f64 { self.latitude.to_degrees() }
    pub fn distance_km(&self) -> f64 { self.distance * 149597870.7 }
}
#[derive(Debug, Clone)]
pub struct LunarPosition { pub epoch: f64, pub longitude: f64, pub latitude: f64, pub distance: f64, pub velocity_x: f64, pub velocity_y: f64, pub velocity_z: f64 }
impl LunarPosition {
    pub fn longitude_deg(&self) -> f64 { self.longitude.to_degrees() }
    pub fn latitude_deg(&self) -> f64 { self.latitude.to_degrees() }
    pub fn distance_km(&self) -> f64 { self.distance * 149597870.7 }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_solar_position() {
        let ephemeris = match JplEphemeris::new(JplEphemerisType::DE441Lite) { Ok(e) => e, Err(_) => return };
        let jd = 2451545.0;
        let sun = ephemeris.solar_position(jd).unwrap();
        assert!((sun.distance - 1.0).abs() < 0.02);
        println!("J2000.0 太阳位置：{:.6}°", sun.longitude_deg());
    }
}
