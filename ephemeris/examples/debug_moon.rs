// 月球计算详细对比调试
// 用于分析 JPL DE441 与 Swiss Ephemeris 在月球计算上的差异

use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use std::process::Command;

/// 运行 swetest 获取详细数据
fn run_swetest_detailed(jd: f64) -> Result<String, String> {
    let swetest_path = "/home/shey/Codes/my/ephemeris/swetest";
    let ephe_path = "/home/shey/Codes/my/ephemeris/ephe";

    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -j{} -p1 -fTPly -n1 -head",
        ephe_path, swetest_path, jd
    );

    let output = Command::new("bash")
        .args(&["-c", &cmd_str])
        .output()
        .map_err(|e| format!("无法执行 swetest: {}", e))?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 运行 swetest 获取地心距
fn run_swetest_distance(jd: f64) -> Result<f64, String> {
    let swetest_path = "/home/shey/Codes/my/ephemeris/swetest";
    let ephe_path = "/home/shey/Codes/my/ephemeris/ephe";

    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -j{} -p1 -fPp -n1 -head",
        ephe_path, swetest_path, jd
    );

    let output = Command::new("bash")
        .args(&["-c", &cmd_str])
        .output()
        .map_err(|e| format!("无法执行 swetest: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let line = stdout.lines()
        .find(|l| !l.is_empty() && !l.starts_with("date") && !l.starts_with("UT:"))
        .ok_or_else(|| format!("未找到数据行：{}", stdout))?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    // Pp 格式：黄经 黄纬 距离 (AU)
    let dist_au = parts.get(2).and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| format!("解析距离失败：{}", line))?;

    Ok(dist_au)
}

fn main() {
    // 测试公元前 987 年的一个点（POOR 测试点）
    // JD 计算：公元前 987 年 ≈ JD 1355804.5
    let test_jd = 1355804.5 + 100.0; // 稍微偏移一点

    println!("\n{}", "=".repeat(120));
    println!("月球计算详细对比调试");
    println!("测试 JD: {}", test_jd);
    println!("{}", "=".repeat(120));

    // 1. Swiss Ephemeris 数据
    match run_swetest_detailed(test_jd) {
        Ok(output) => {
            println!("\nSwiss Ephemeris 输出:");
            println!("{}", output);
        },
        Err(e) => println!("Swiss Ephemeris 错误：{}", e),
    }

    // 2. Swiss Ephemeris 距离
    match run_swetest_distance(test_jd) {
        Ok(dist_au) => {
            println!("\nSwiss Ephemeris 地心距：{} AU ({} km)", dist_au, dist_au * 149597870.7);
        },
        Err(e) => println!("距离获取错误：{}", e),
    }

    // 3. JPL DE441 数据
    match JplEphemeris::new(JplEphemerisType::DE441Lite) {
        Ok(jpl) => {
            match jpl.lunar_position(test_jd) {
                Ok(pos) => {
                    println!("\nJPL DE441 数据:");
                    println!("  黄经：{:.6}°", pos.longitude.to_degrees());
                    println!("  黄纬：{:.6}°", pos.latitude.to_degrees());
                    println!("  距离：{:.6} AU ({} km)", pos.distance, pos.distance * 149597870.7);
                },
                Err(e) => println!("JPL 位置错误：{}", e),
            }
        },
        Err(e) => println!("JPL 加载错误：{}", e),
    }

    // 4. 对比多个点
    println!("\n{}", "=".repeat(120));
    println!("多点对比:");
    println!("{:<15} {:>15} {:>15} {:>15} {:>15}",
             "JD", "Swiss Lon(°)", "JPL Lon(°)", "Diff(arcsec)", "Dist Diff(%)");
    println!("{}", "=".repeat(120));

    let test_jds = vec![
        1355804.5,  // 公元前 1000 年
        1355904.5,  // 偏移 100 天
        1356004.5,  // 偏移 200 天
        1720000.0,  // 公元 1000 年
        2451545.0,  // J2000
    ];

    for jd in test_jds {
        let swiss_dist = run_swetest_distance(jd).unwrap_or(0.0);
        
        if let Ok(jpl) = JplEphemeris::new(JplEphemerisType::DE441Lite) {
            if let Ok(pos) = jpl.lunar_position(jd) {
                let cmd_str = format!(
                    "SE_EPHE_PATH=/home/shey/Codes/my/ephemeris/ephe /home/shey/Codes/my/ephemeris/swetest -j{} -p1 -fPlong -n1 -head",
                    jd
                );
                if let Ok(output) = Command::new("bash").args(&["-c", &cmd_str]).output() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    if let Some(line) = stdout.lines().find(|l| !l.is_empty() && !l.starts_with("date") && !l.starts_with("UT:")) {
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if let Some(swiss_lon) = parts.get(1).and_then(|s| s.parse::<f64>().ok()) {
                            let mut diff = (swiss_lon - pos.longitude.to_degrees()).abs();
                            if diff > 180.0 { diff = 360.0 - diff; }
                            let diff_arcsec = diff * 3600.0;
                            
                            let dist_diff_pct = if swiss_dist > 0.0 {
                                (swiss_dist - pos.distance).abs() / swiss_dist * 100.0
                            } else { 0.0 };
                            
                            println!("{:<15.1} {:>15.6} {:>15.6} {:>15.4} {:>15.4}",
                                     jd, swiss_lon, pos.longitude.to_degrees(), diff_arcsec, dist_diff_pct);
                        }
                    }
                }
            }
        }
    }
    println!("{}", "=".repeat(120));
}
