// 测试 anise 光行差修正
use anise::prelude::*;
use anise::constants::frames::{EARTH_J2000, SUN_J2000};

fn main() {
    println!("=== 测试 anise 光行差修正 ===\n");
    
    let mut almanac = Almanac::default();
    
    // 加载 BSP 文件
    let bsp_path = "bsp/441/de441_3000bc_3000ad.bsp";
    match SPK::load(bsp_path) {
        Ok(spk) => {
            println!("✓ 加载 BSP 成功");
            almanac = almanac.with_spk(spk);
        }
        Err(e) => {
            println!("✗ 加载失败：{}", e);
            return;
        }
    }
    
    let jd = 2451545.0;
    let epoch = Epoch::from_jde_tdb(jd);
    
    // 1. 无光行差修正
    println!("\n1. 无光行差修正 (Aberration::NONE):");
    match almanac.translate(SUN_J2000, EARTH_J2000, epoch, Aberration::NONE) {
        Ok(state) => {
            let x = state.radius_km.x;
            let y = state.radius_km.y;
            let lon = y.atan2(x).to_degrees();
            println!("   黄经：{:.6}°", lon);
        }
        Err(e) => println!("   失败：{}", e),
    }
    
    // 2. 光行差修正 (LT - 非收敛光时)
    println!("\n2. 光行差修正 (Aberration::LT):");
    match almanac.translate(SUN_J2000, EARTH_J2000, epoch, Aberration::LT) {
        Ok(state) => {
            let x = state.radius_km.x;
            let y = state.radius_km.y;
            let lon = y.atan2(x).to_degrees();
            println!("   黄经：{:.6}°", lon);
        }
        Err(e) => println!("   失败：{}", e),
    }
    
    // 3. 光行差修正 (CN_S - 收敛光时 + 恒星)
    println!("\n3. 光行差修正 (Aberration::CN_S):");
    match almanac.translate(SUN_J2000, EARTH_J2000, epoch, Aberration::CN_S) {
        Ok(state) => {
            let x = state.radius_km.x;
            let y = state.radius_km.y;
            let lon = y.atan2(x).to_degrees();
            println!("   黄经：{:.6}°", lon);
        }
        Err(e) => println!("   失败：{}", e),
    }
    
    // 对比 Swiss Ephemeris
    println!("\n4. Swiss Ephemeris (参考):");
    println!("   黄经：260.06° (视位置)");
}
