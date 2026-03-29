// JPL 历表与 Swiss Ephemeris 对比测试

use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::internal::planet::Planet;
use std::process::Command;

/// 天体位置数据
#[derive(Debug)]
struct BodyPosition {
    name: String,
    jd: f64,
    ecl_lon: f64, // 度
}

/// 运行 swetest 获取位置数据
fn run_swetest(swetest_date: &str, time_str: &str, planet_flag: &str) -> Option<BodyPosition> {
    let swetest_path = "/home/shey/Codes/my/ephemeris/swetest";
    let ephe_path = "/home/shey/Codes/my/ephemeris/ephe";
    
    // -ut 指定时刻，确保时间对齐
    // -fPlong 输出黄经
    // -head 隐藏页眉
    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -b{} -ut{} -p{} -fPlong -n1 -head",
        ephe_path, swetest_path, swetest_date, time_str, planet_flag
    );
    
    let cmd = Command::new("bash")
        .args(&["-c", &cmd_str])
        .output();
    
    match cmd {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).ok()?;
            parse_swetest_output(&stdout)
        }
        Err(_) => None,
    }
}

fn parse_swetest_output(output: &str) -> Option<BodyPosition> {
    let line = output.lines().next()?;
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() < 2 { return None; }
    
    let name = parts[0].to_string();
    let ecl_lon = parts[1].parse::<f64>().ok()?;
    
    Some(BodyPosition {
        name,
        jd: 0.0, // 此处 JD 仅作占位
        ecl_lon,
    })
}

fn compare_positions(name: &str, swetest_lon: f64, jpl_lon: f64) {
    let mut diff = (swetest_lon - jpl_lon).abs();
    if diff > 180.0 { diff = 360.0 - diff; }
    
    // 转换为角秒 (1 degree = 3600 arcseconds)
    let diff_arcsec = diff * 3600.0;
    
    println!("{:<12} {:>15.6} {:>15.6} {:>15.3} arcsec", 
             name, swetest_lon, jpl_lon, diff_arcsec);
}

#[test]
fn test_jpl_vs_swiss_ephemeris() {
    println!("\n{}", "=".repeat(80));
    println!("{:<12} {:>15} {:>15} {:>15}", "Planet", "Swiss Eph(°)", "JPL DE441(°)", "Diff (Arcsec)");
    println!("{}", "=".repeat(80));
    
    let jpl = JplEphemeris::new(JplEphemerisType::DE441Lite).expect("Failed to load JPL");
    
    // 测试时刻：J2000.0 (2000-01-01 12:00:00 UT)
    let test_date = "01.01.2000";
    let test_time = "12:00:00";
    let jd = 2451545.0;
    
    let planets = vec![
        (Planet::Sun, "0", "Sun"),
        (Planet::Moon, "1", "Moon"),
        (Planet::Mercury, "2", "Mercury"),
        (Planet::Venus, "3", "Venus"),
        (Planet::Mars, "4", "Mars"),
        (Planet::Jupiter, "5", "Jupiter"),
        (Planet::Saturn, "6", "Saturn"),
    ];
    
    for (planet, flag, name) in planets {
        if let Some(swetest_pos) = run_swetest(test_date, test_time, flag) {
            if let Ok(jpl_pos) = jpl.planet_position(planet, jd) {
                compare_positions(name, swetest_pos.ecl_lon, jpl_pos.longitude_deg());
            }
        }
    }
    
    println!("{}", "=".repeat(80));
    println!("Note: Swiss Ephemeris is using Moshier approximation in this environment.");
}
