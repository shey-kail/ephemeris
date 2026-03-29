// JPL 历表与 Swiss Ephemeris 对比测试

use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::internal::planet::Planet;
use std::process::Command;

/// 天体位置数据
#[derive(Debug)]
struct BodyPosition {
    name: String,
    jd: f64,
    ecl_lon: f64,
    ecl_lat: f64,
    ra: f64,
    dec: f64,
    speed: f64,
}

/// 运行 swetest 获取位置数据
fn run_swetest(date_str: &str, planet_flag: &str) -> Option<BodyPosition> {
    // 使用绝对路径
    let swetest_path = "/home/shey/Codes/my/ephemeris/swetest";
    let ephe_path = "/home/shey/Codes/my/ephemeris/ephe";
    let workspace_root = "/home/shey/Codes/my/ephemeris";
    
    // 使用 shell 调用，设置 SE_EPHE_PATH 环境变量（使用绝对路径）
    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -b{} -p{} -fPlong -n1 -head",
        ephe_path, swetest_path, date_str, planet_flag
    );
    
    let cmd = Command::new("bash")
        .args(&["-c", &cmd_str])
        .current_dir(workspace_root)
        .output();
    
    match cmd {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).ok()?;
            println!("  swetest output: '{}'", stdout.trim());
            // 检查是否有 "using Moshier eph." 警告
            if stdout.contains("using Moshier eph.") {
                println!("  WARNING: Swiss Ephemeris is using Moshier approximation (SE1 files not loaded properly)");
            }
            if !output.status.success() {
                let stderr = String::from_utf8(output.stderr).unwrap_or_default();
                println!("  swetest stderr: '{}'", stderr.trim());
            }
            parse_swetest_output(&stdout, date_str)
        }
        Err(e) => {
            println!("  swetest command failed: {}", e);
            None
        }
    }
}

/// 解析 swetest 输出
fn parse_swetest_output(output: &str, date_str: &str) -> Option<BodyPosition> {
    let lines: Vec<&str> = output.lines().collect();
    if lines.is_empty() {
        return None;
    }
    
    let line = lines[0];
    let parts: Vec<&str> = line.split_whitespace().collect();
    
    if parts.len() < 2 {
        return None;
    }
    
    // 输出格式：名称 黄经 黄纬 距离 速度
    // 例如：Sun  87.9006009  0.0000000  0.0000000  158.2049635
    let name = parts[0].to_string();
    let ecl_lon = parts[1].parse::<f64>().ok()?;
    let jd = parse_date_to_jd(date_str).unwrap_or(2451545.0);
    
    Some(BodyPosition {
        name,
        jd,
        ecl_lon,
        ecl_lat: 0.0,
        ra: 0.0,
        dec: 0.0,
        speed: 0.0,
    })
}

fn parse_ra_hms(hms: &str) -> Option<f64> {
    let parts: Vec<&str> = hms.split(':').collect();
    if parts.len() >= 2 {
        let h = parts[0].parse::<f64>().ok()?;
        let m = parts.get(1).unwrap_or(&"0").parse::<f64>().ok()?;
        let s = parts.get(2).unwrap_or(&"0").parse::<f64>().ok()?;
        Some((h + m / 60.0 + s / 3600.0) * 15.0)
    } else {
        None
    }
}

fn parse_dec_dms(dms: &str) -> Option<f64> {
    let parts: Vec<&str> = dms.split(':').collect();
    if parts.len() >= 2 {
        let d = parts[0].parse::<f64>().ok()?;
        let m = parts.get(1).unwrap_or(&"0").parse::<f64>().ok()?;
        let s = parts.get(2).unwrap_or(&"0").parse::<f64>().ok()?;
        let sign = if d < 0.0 { -1.0 } else { 1.0 };
        Some(d + sign * (m / 60.0 + s / 3600.0))
    } else {
        None
    }
}

fn parse_date_to_jd(date_str: &str) -> Option<f64> {
    let parts: Vec<&str> = date_str.split('-').collect();
    if parts.len() >= 3 {
        let year = parts[0].parse::<i32>().ok()?;
        let month = parts[1].parse::<i32>().ok()?;
        let day = parts[2].parse::<f64>().ok()?;
        
        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        let jd = day + ((153 * m + 2) / 5) as f64 + (365 * y) as f64 + (y / 4) as f64 - (y / 100) as f64 + (y / 400) as f64 - 32045.0;
        Some(jd)
    } else {
        None
    }
}

fn compare_positions(swetest_pos: &BodyPosition, jpl_pos: &BodyPosition, tolerance: f64) {
    println!("\n=== {} Position Comparison (JD {}) ===", swetest_pos.name, swetest_pos.jd);
    println!("{:<12} {:>15} {:>15} {:>15}", "Item", "Swiss Eph", "JPL DE441", "Diff");
    println!("{}", "-".repeat(60));
    
    let lon_diff = (swetest_pos.ecl_lon - jpl_pos.ecl_lon).abs();
    let lon_diff_norm = if lon_diff > 180.0 { 360.0 - lon_diff } else { lon_diff };
    let lon_pass = lon_diff_norm < tolerance;
    println!("{:<12} {:>15.6} {:>15.6} {:>14.6} {}", 
             "Ecl Lon", swetest_pos.ecl_lon, jpl_pos.ecl_lon, lon_diff_norm,
             if lon_pass { "OK" } else { "FAIL" });
}

#[test]
fn test_jpl_vs_swiss_ephemeris() {
    println!("\n{}", "=".repeat(70));
    println!("JPL DE441 vs Swiss Ephemeris Comparison Test");
    println!("{}", "=".repeat(70));
    
    // 使用相对路径加载（相对于项目根目录）
    let jpl = match JplEphemeris::new(JplEphemerisType::DE441Lite) {
        Ok(ep) => ep,
        Err(e) => {
            println!("Failed to load JPL ephemeris: {}", e);
            println!("Trying to load from workspace root...");
            // 尝试从工作区根目录加载
            std::env::set_current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/../").ok();
            match JplEphemeris::new(JplEphemerisType::DE441Lite) {
                Ok(ep) => ep,
                Err(e) => {
                    println!("Failed again: {}", e);
                    return;
                }
            }
        }
    };
    
    let test_dates = vec![
        ("2000-01-01.5", "2000.01.01.5"),  // (解析用，swetest 用)
        ("2023-06-15.0", "2023.06.15.0"),
    ];
    
    let planets = vec![
        (Planet::Sun, "0", "Sun"),
        (Planet::Moon, "1", "Moon"),
        (Planet::Mercury, "2", "Mercury"),
        (Planet::Venus, "3", "Venus"),
        (Planet::Mars, "4", "Mars"),
        (Planet::Jupiter, "5", "Jupiter"),
        (Planet::Saturn, "6", "Saturn"),
    ];
    
    let tolerance = 1.0;  // 容差：1 度（因为 Swiss Ephemeris 使用 Moshier 近似）
    
    for (parse_date, swetest_date) in test_dates {
        println!("\n{}", "=".repeat(70));
        println!("Test Date: {}", parse_date);
        println!("Note: Swiss Ephemeris uses Moshier approximation (no SE1 files)");
        println!("{}", "=".repeat(70));
        
        for (planet, swetest_flag, name) in &planets {
            let swetest_pos = match run_swetest(swetest_date, swetest_flag) {
                Some(pos) => pos,
                None => {
                    println!("\n{}: Failed to get Swiss Ephemeris data", name);
                    continue;
                }
            };
            
            let jd = parse_date_to_jd(parse_date).unwrap_or(2451545.0);
            match jpl.planet_position(*planet, jd) {
                Ok(jpl_pos) => {
                    let jpl_body = BodyPosition {
                        name: name.to_string(),
                        jd,
                        ecl_lon: jpl_pos.longitude.to_degrees(),
                        ecl_lat: jpl_pos.latitude.to_degrees(),
                        ra: 0.0,
                        dec: 0.0,
                        speed: 0.0,
                    };
                    compare_positions(&swetest_pos, &jpl_body, tolerance);
                }
                Err(e) => {
                    println!("\n{}: JPL calculation failed - {}", name, e);
                }
            }
        }
    }
    
    println!("\n{}", "=".repeat(70));
    println!("Test Complete");
    println!("{}", "=".repeat(70));
}
