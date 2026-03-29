// 计算模式示例：演示简单模式和精准模式的使用

use rust_ephemeris::internal::mode::{EphemerisCalculator, CalculationMode};

fn main() {
    println!("=== 天文计算模式示例 ===\n");
    
    // ==================== 简单模式 ====================
    println!("【简单模式】使用 Moshier 近似算法");
    println!("特点：速度快，无需历表文件，精度约 1-2 秒\n");
    
    let simple_calc = EphemerisCalculator::simple();
    println!("模式：{:?}", simple_calc.mode());
    
    // 计算太阳位置
    let jd = 2451545.0; // J2000.0
    match simple_calc.solar_position(jd) {
        Ok(pos) => {
            match pos {
                rust_ephemeris::internal::mode::SolarPositionResult::Simple(sun) => {
                    println!("太阳位置 (J2000.0):");
                    println!("  黄经：{:.6}°", sun.longitude_deg());
                    println!("  距离：{:.6} AU", sun.distance);
                }
                _ => {}
            }
        }
        Err(e) => println!("计算失败：{}", e),
    }
    
    // 计算朔时刻
    match simple_calc.find_new_moon(jd, Some(30.0)) {
        Ok(new_moon_jd) => {
            println!("朔时刻：JD {:.6} (J2000 后 {:.2f} 天)", new_moon_jd, new_moon_jd - jd);
        }
        Err(e) => println!("计算失败：{}", e),
    }
    
    println!();
    
    // ==================== 精准模式 ====================
    println!("【精准模式】使用 JPL DE441 历表");
    println!("特点：精度高（< 0.1 秒），需要历表文件，速度较慢\n");
    
    match EphemerisCalculator::precise() {
        Ok(precise_calc) => {
            println!("模式：{:?}", precise_calc.mode());
            
            // 计算太阳位置
            match precise_calc.solar_position(jd) {
                Ok(pos) => {
                    match pos {
                        rust_ephemeris::internal::mode::SolarPositionResult::Precise(sun) => {
                            println!("太阳位置 (J2000.0):");
                            println!("  黄经：{:.6}°", sun.longitude_deg());
                            println!("  距离：{:.6} AU", sun.distance);
                        }
                        _ => {}
                    }
                }
                Err(e) => println!("计算失败：{}", e),
            }
            
            // 计算朔时刻
            match precise_calc.find_new_moon(jd, Some(30.0)) {
                Ok(new_moon_jd) => {
                    println!("朔时刻：JD {:.6} (J2000 后 {:.2f} 天)", new_moon_jd, new_moon_jd - jd);
                }
                Err(e) => println!("计算失败：{}", e),
            }
            
            // 计算望时刻
            match precise_calc.find_full_moon(jd, Some(30.0)) {
                Ok(full_moon_jd) => {
                    println!("望时刻：JD {:.6} (J2000 后 {:.2f} 天)", full_moon_jd, full_moon_jd - jd);
                }
                Err(e) => println!("计算失败：{}", e),
            }
            
            // 计算节气
            let term_names = ["春分", "夏至", "秋分", "冬至"];
            let term_indices = [0, 3, 2, 1]; // 简化：只计算 4 个主要节气
            
            println!("节气时刻:");
            for (name, index) in term_names.iter().zip(term_indices.iter()) {
                match precise_calc.find_solar_term(jd, *index) {
                    Ok(term_jd) => {
                        println!("  {}: JD {:.6}", name, term_jd);
                    }
                    Err(e) => println!("  {} 计算失败：{}", name, e),
                }
            }
        }
        Err(e) => {
            println!("无法加载精准模式：{}", e);
            println!("请确保 bsp/441/ 目录下存在 de441_part-1.bsp 和 de441_part-2.bsp 文件");
        }
    }
    
    println!();
    
    // ==================== 模式对比 ====================
    println!("【模式对比】");
    println!("简单模式：");
    println!("  ✓ 无需外部文件");
    println!("  ✓ 计算速度快（< 1 μs）");
    println!("  ✓ 适用于农历计算、一般天文应用");
    println!("  ✗ 精度约 1-2 秒");
    println!();
    println!("精准模式：");
    println!("  ✓ 精度高（< 0.1 秒）");
    println!("  ✓ 适用于高精度天文计算、科学研究");
    println!("  ✗ 需要 BSP 历表文件（约 500 MB）");
    println!("  ✗ 计算速度较慢（~10 μs）");
}
