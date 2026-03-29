// 使用 anise 生成朔望和节气的高精度参考值
//
// 运行：cargo run --example generate_reference --release

use hifitime::{Epoch, TimeScale, Unit};
use std::f64::consts::PI;

// 引入当前项目的计算函数
use rust_ephemeris::internal::ephemeris::{moon_a_lon_t, earth_lon, solor_a_lon_t};

/// 计算太阳黄经（使用当前算法）
fn solar_longitude_approx(t: f64) -> f64 {
    // t 是相对于 J2000 的儒略世纪
    earth_lon(t, -1) + PI
}

/// 计算月球黄经（使用当前算法）
fn lunar_longitude_approx(t: f64) -> f64 {
    moon_a_lon_t(0.0) * 36525.0 // 这里需要正确实现
}

/// 使用二分法搜索朔时刻
fn find_new_moon_brent(start_epoch: Epoch, max_days: f64) -> Epoch {
    let search_func = |epoch: Epoch| -> f64 {
        let t = (epoch.to_tt_duration() - Epoch::from_gregorian_tt(2000, 1, 1, 12, 0, 0).to_tt_duration()).to_unit(Unit::Day) / 36525.0;
        
        // 简化：使用近似公式
        let sun_lon = solar_longitude_approx(t);
        let moon_lon = lunar_longitude_approx(t);
        
        let mut diff = moon_lon - sun_lon;
        while diff < 0.0 {
            diff += 2.0 * PI;
        }
        while diff > 2.0 * PI {
            diff -= 2.0 * PI;
        }
        diff - PI // 在朔时应该接近 0
    };
    
    // 简单的二分搜索
    let mut t1 = start_epoch;
    let mut t2 = start_epoch + Unit::Day * max_days;
    
    let f1 = search_func(t1);
    let f2 = search_func(t2);
    
    if f1 * f2 > 0.0 {
        return start_epoch; // 没有符号变化
    }
    
    for _ in 0..50 {
        let mid = t1 + (t2 - t1) * 0.5;
        let f_mid = search_func(mid);
        
        if f_mid.abs() < 1e-10 {
            return mid;
        }
        
        if f1 * f_mid < 0.0 {
            t2 = mid;
        } else {
            t1 = mid;
        }
    }
    
    t1 + (t2 - t1) * 0.5
}

fn main() {
    println!("生成朔望和节气参考值...");
    
    // 测试用例 1：so_high 测试
    let w1 = 1727.8759594743863;
    let expected1 = 8126.101574259753;
    
    // 测试用例 2：so_low 测试
    let w2 = -4354.247417875453;
    let expected2 = -20458.974805811675;
    
    // 测试用例 3：qi_hight 测试
    let w3 = 58.119464091411174;
    let expected3 = 3093.8331491526683;
    
    println!("\n=== 当前测试用例分析 ===");
    println!("测试 1 (so_high): w={}, 期望值={}", w1, expected1);
    println!("测试 2 (so_low): w={}, 期望值={}", w2, expected2);
    println!("测试 3 (qi_hight): w={}, 期望值={}", w3, expected3);
    
    // 分析 w 的含义
    // w 应该是太阳或月球的平黄经（弧度）
    // 对于朔：w = 月球平黄经
    // 对于节气：w = 太阳平黄经
    
    println!("\n=== W 值分析 ===");
    println!("w1/PI = {}", w1 / PI);
    println!("w2/PI = {}", w2 / PI);
    println!("w3/PI = {}", w3 / PI);
    
    // 计算对应的儒略日
    println!("\n=== 对应的儒略日 ===");
    println!("expected1 (JD) = {}", expected1 + 2451545.0);
    println!("expected2 (JD) = {}", expected2 + 2451545.0);
    println!("expected3 (JD) = {}", expected3 + 2451545.0);
    
    // 分析误差来源
    println!("\n=== 误差分析 ===");
    
    // 对于 so_low，使用 moon_a_lon_t 计算
    let t_so_low = moon_a_lon_t(w2);
    let jd_so_low = t_so_low * 36525.0;
    println!("so_low: moon_a_lon_t 返回 t={}, JD={}", t_so_low, jd_so_low);
    println!("so_low: 期望 JD={}, 误差={}", expected2, (jd_so_low - expected2).abs());
    
    // 对于 qi_hight，使用 solor_a_lon_t 计算
    let t_qi = solor_a_lon_t(w3);
    let jd_qi = t_qi * 36525.0;
    println!("qi_hight: solor_a_lon_t 返回 t={}, JD={}", t_qi, jd_qi);
    println!("qi_hight: 期望 JD={}, 误差={}", expected3, (jd_qi - expected3).abs());
}
