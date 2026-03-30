use anise::prelude::*;
use anise::constants::frames::{EARTH_J2000, MOON_J2000};
use std::path::Path;

fn main() {
    let workspace_root = "/home/shey/Codes/my/ephemeris";
    let path = workspace_root.to_string() + "/bsp/441/de441_3000bc_3000ad.bsp";
    
    let mut almanac = Almanac::default();
    let spk = SPK::load(&path).expect("加载 BSP 失败");
    almanac = almanac.with_spk(spk);
    
    let jd_tdb = 1355804.5;
    let epoch = Epoch::from_jde_tdb(jd_tdb);
    
    // 获取月球相对于地球的 ICRS (J2000) 状态
    let state = almanac.translate(MOON_J2000, EARTH_J2000, epoch, Aberration::NONE)
        .expect("anise translate 失败");
        
    let pos_km = state.radius_km;
    const KM_PER_AU: f64 = 149597870.7;
    
    println!("Anise ICRS Moon Position (AU):");
    println!("X: {:.12}", pos_km.x / KM_PER_AU);
    println!("Y: {:.12}", pos_km.y / KM_PER_AU);
    println!("Z: {:.12}", pos_km.z / KM_PER_AU);
}
