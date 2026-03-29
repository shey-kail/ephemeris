// 验证 JPL DE441 历表计算结果
// 对比 JPL DE441 与近似算法的计算结果

use rust_ephemeris::internal::mode::{EphemerisCalculator, CalculationMode};
use rust_ephemeris::internal::planet::Planet;

fn main() {
    println!("=== JPL DE441 历表验证 ===\n");
    
    // 测试日期：J2000.0
    let jd = 2451545.0;
    println!("测试日期：J2000.0 (JD {})", jd);
    println!("对应公历：2000 年 1 月 1 日 12:00 TT\n");
    
    // 创建精准模式计算器（使用 DE441 精简版）
    match EphemerisCalculator::new(rust_ephemeris::internal::mode::EphemerisConfig {
        mode: rust_ephemeris::internal::mode::CalculationMode::Precise {
            ephemeris_type: rust_ephemeris::internal::jpl_ephemeris::JplEphemerisType::DE441Lite,
        },
        ..Default::default()
    }) {
        Ok(calc) => {
            println!("✓ 成功加载 JPL DE441 历表\n");
            
            // 测试太阳位置
            println!("=== 太阳位置 ===");
            match calc.solar_position(jd) {
                Ok(pos) => {
                    match pos {
                        rust_ephemeris::internal::mode::SolarPositionResult::Precise(sun) => {
                            println!("  黄经：{:.6}°", sun.longitude_deg());
                            println!("  黄纬：{:.6}°", sun.latitude_deg());
                            println!("  距离：{:.6} AU ({:.0} km)", sun.distance, sun.distance_km());
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("  计算失败：{}", e),
            }
            
            // 测试月球位置
            println!("\n=== 月球位置 ===");
            match calc.lunar_position(jd) {
                Ok(pos) => {
                    match pos {
                        rust_ephemeris::internal::mode::LunarPositionResult::Precise(moon) => {
                            println!("  黄经：{:.6}°", moon.longitude_deg());
                            println!("  黄纬：{:.6}°", moon.latitude_deg());
                            println!("  距离：{:.6} AU ({:.0} km)", moon.distance, moon.distance_km());
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("  计算失败：{}", e),
            }
            
            // 测试行星位置
            println!("\n=== 行星位置 ===");
            let planets = vec![
                (Planet::Mercury, "水星"),
                (Planet::Venus, "金星"),
                (Planet::Mars, "火星"),
                (Planet::Jupiter, "木星"),
                (Planet::Saturn, "土星"),
            ];
            
            for (planet, name) in planets {
                match calc.planet_position(planet, jd) {
                    Ok(pos) => {
                        match pos {
                            rust_ephemeris::internal::mode::PlanetPositionResult::Precise(p) => {
                                println!("  {}: 黄经={:.6}° 黄纬={:.6}° 距离={:.6} AU", 
                                         name, p.longitude_deg(), p.latitude_deg(), p.distance);
                            }
                            _ => {}
                        }
                    }
                    Err(e) => println!("  {}: 计算失败 - {}", name, e),
                }
            }
            
            // 测试朔望计算
            println!("\n=== 朔望计算 ===");
            match calc.find_new_moon(jd, Some(30.0)) {
                Ok(new_moon_jd) => {
                    println!("  朔时刻：JD {:.6} (J2000 后 {:.2} 天)", new_moon_jd, new_moon_jd - jd);
                }
                Err(e) => println!("  朔计算失败：{}", e),
            }
            
            match calc.find_full_moon(jd, Some(30.0)) {
                Ok(full_moon_jd) => {
                    println!("  望时刻：JD {:.6} (J2000 后 {:.2} 天)", full_moon_jd, full_moon_jd - jd);
                }
                Err(e) => println!("  望计算失败：{}", e),
            }
            
            // 测试节气计算
            println!("\n=== 节气计算 ===");
            let terms = vec![
                (0, "春分"),
                (3, "夏至"),
                (2, "秋分"),
                (1, "冬至"),
            ];
            
            for (index, name) in terms {
                match calc.find_solar_term(jd, index) {
                    Ok(term_jd) => {
                        println!("  {}: JD {:.6}", name, term_jd);
                    }
                    Err(e) => println!("  {} 计算失败：{}", name, e),
                }
            }
        }
        Err(e) => {
            println!("✗ 无法加载 JPL DE441 历表：{}", e);
            println!("请确保 bsp/441/ 目录下存在 de441_part-1.bsp 和 de441_part-2.bsp 文件");
        }
    }
    
    // 对比简单模式
    println!("\n\n=== 简单模式对比 (Moshier 近似) ===");
    let simple_calc = EphemerisCalculator::simple();
    
    match simple_calc.solar_position(jd) {
        Ok(pos) => {
            match pos {
                rust_ephemeris::internal::mode::SolarPositionResult::Simple(sun) => {
                    println!("太阳黄经：{:.6}°", sun.longitude_deg());
                }
                _ => {}
            }
        }
        _ => {}
    }
}
