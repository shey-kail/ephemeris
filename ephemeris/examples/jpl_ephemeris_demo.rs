// JPL 历表示例：演示如何使用高精度历表计算朔望和节气

use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};

fn main() {
    println!("=== JPL 历表示例 ===\n");
    
    // 1. 创建 JPL 历表计算器
    let ephemeris = match JplEphemeris::new(JplEphemerisType::DE441) {
        Ok(ep) => ep,
        Err(e) => {
            println!("无法加载 JPL 历表：{}", e);
            println!("请确保 bsp/441/ 目录下存在 de441_part-1.bsp 和 de441_part-2.bsp 文件");
            return;
        }
    };
    
    println!("已加载 JPL DE441 历表");
    println!("覆盖范围：{} 年到 {} 年\n", ephemeris.year_range().0, ephemeris.year_range().1);
    
    // 2. 计算 J2000.0 时刻的太阳位置
    let jd_j2000 = 2451545.0;
    match ephemeris.solar_position(jd_j2000) {
        Ok(sun) => {
            println!("J2000.0 太阳位置:");
            println!("  黄经：{:.6}°", sun.longitude_deg());
            println!("  黄纬：{:.6}°", sun.latitude_deg());
            println!("  距离：{:.6} AU ({:.0f} km)", sun.distance, sun.distance_km());
        }
        Err(e) => println!("计算太阳位置失败：{}", e),
    }
    println!();
    
    // 3. 计算 J2000.0 时刻的月球位置
    match ephemeris.lunar_position(jd_j2000) {
        Ok(moon) => {
            println!("J2000.0 月球位置:");
            println!("  黄经：{:.6}°", moon.longitude_deg());
            println!("  黄纬：{:.6}°", moon.latitude_deg());
            println!("  距离：{:.6} AU ({:.0f} km)", moon.distance, moon.distance_km());
        }
        Err(e) => println!("计算月球位置失败：{}", e),
    }
    println!();
    
    // 4. 计算 2000 年附近的朔（新月）时刻
    println!("计算 2000 年附近的朔望时刻...");
    let jd_2000 = 2451545.0;
    
    match ephemeris.find_new_moon(jd_2000, Some(30.0)) {
        Ok(jd) => {
            let days_from_j2000 = jd - jd_2000;
            println!("  朔时刻：JD {:.6} (J2000 后 {:.2f} 天)", jd, days_from_j2000);
        }
        Err(e) => println!("计算朔时刻失败：{}", e),
    }
    
    match ephemeris.find_full_moon(jd_2000, Some(30.0)) {
        Ok(jd) => {
            let days_from_j2000 = jd - jd_2000;
            println!("  望时刻：JD {:.6} (J2000 后 {:.2f} 天)", jd, days_from_j2000);
        }
        Err(e) => println!("计算望时刻失败：{}", e),
    }
    println!();
    
    // 5. 计算 2000 年的节气
    println!("计算 2000 年的节气...");
    let term_names = [
        "春分", "清明", "谷雨", "立夏", "小满", "芒种",
        "夏至", "小暑", "大暑", "立秋", "处暑", "白露",
        "秋分", "寒露", "霜降", "立冬", "小雪", "大雪",
        "冬至", "小寒", "大寒", "立春", "雨水", "惊蛰"
    ];
    
    for (i, name) in term_names.iter().enumerate() {
        match ephemeris.find_solar_term(jd_2000, i) {
            Ok(jd) => {
                let days_from_j2000 = jd - jd_2000;
                println!("  {}: JD {:.6} (J2000 后 {:.2f} 天)", name, jd, days_from_j2000);
            }
            Err(e) => println!("  {} 计算失败：{}", name, e),
        }
    }
}
