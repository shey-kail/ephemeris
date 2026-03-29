// 测试 BSP 文件加载
use anise::prelude::*;

fn main() {
    println!("=== 测试 BSP 文件加载 ===\n");
    
    let mut almanac = Almanac::default();
    
    // 加载 DE441 精简版（-3000BC ~ 3000AD）
    let bsp_path = "bsp/441/de441_3000bc_3000ad.bsp";
    match SPK::load(bsp_path) {
        Ok(spk) => {
            println!("✓ 成功加载 {}", bsp_path);
            almanac = almanac.with_spk(spk);
            println!("  已加载的 SPK 数量：{}", almanac.num_loaded_spk());
        }
        Err(e) => {
            println!("✗ 加载失败 {}: {}", bsp_path, e);
        }
    }
    
    // 测试太阳位置计算
    println!("\n=== 测试太阳位置计算 ===");
    use anise::constants::frames::{EARTH_J2000, SUN_J2000};
    
    let jd = 2451545.0;
    let epoch = Epoch::from_jde_tdb(jd);
    
    match almanac.translate(SUN_J2000, EARTH_J2000, epoch, None) {
        Ok(state) => {
            let x = state.radius_km.x;
            let y = state.radius_km.y;
            let r = (x*x + y*y).sqrt();
            let lon = y.atan2(x);
            println!("✓ 太阳位置计算成功");
            println!("  黄经：{:.6}°", lon.to_degrees());
            println!("  距离：{:.6} km", r);
        }
        Err(e) => {
            println!("✗ 计算失败：{}", e);
        }
    }
    
    // 测试月球位置计算
    println!("\n=== 测试月球位置计算 ===");
    use anise::constants::frames::MOON_J2000;
    
    match almanac.translate(MOON_J2000, EARTH_J2000, epoch, None) {
        Ok(state) => {
            let x = state.radius_km.x;
            let y = state.radius_km.y;
            let r = (x*x + y*y).sqrt();
            let lon = y.atan2(x);
            println!("✓ 月球位置计算成功");
            println!("  黄经：{:.6}°", lon.to_degrees());
            println!("  距离：{:.6} km", r);
        }
        Err(e) => {
            println!("✗ 计算失败：{}", e);
        }
    }
}
