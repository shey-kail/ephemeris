// JPL DE431 历表与 Swiss Ephemeris 跨越 6000 年的长周期全参数精度对比测试
// 测试范围：公元前 3000 年 至 公元 3000 年
// 测试参数：视黄经、视黄纬、地心距、瞬时速度 (km/s)

use rand::SeedableRng;
use rand::Rng;
use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::internal::planet::Planet;
use std::process::Command;

#[derive(Debug, Default)]
struct SwetestResult {
    lon: f64,
    lat: f64,
    dist: f64,
    speed: f64, // km/s
}

/// 运行 swetest 获取指定 JD 的全方位参数
fn run_swetest_detailed(jd: f64, planet_flag: &str) -> Result<SwetestResult, String> {
    let swetest_path = "/home/shey/Codes/my/ephemeris/swetest";
    let ephe_path = "/home/shey/Codes/my/ephemeris/ephe";

    // -fPlbrs: P(name), l(lon dec), b(lat dec), r(dist AU), s(speed lon deg/day)
    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -j{} -p{} -fPlbrs -n1 -head",
        ephe_path, swetest_path, jd, planet_flag
    );

    let output = Command::new("bash")
        .args(&["-c", &cmd_str])
        .output()
        .map_err(|e| format!("无法执行 swetest: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    let line = stdout.lines()
        .find(|l| !l.is_empty() && !l.contains("date") && !l.contains("UT:") && !l.contains("TT:") && !l.contains("Epsilon"))
        .ok_or_else(|| format!("未找到数据行：{}", stdout))?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    
    let lon = parts.get(1).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let lat = parts.get(2).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);
    let mut dist_str = parts.get(3).unwrap_or(&"0.0").to_string();
    let speed = parts.get(4).and_then(|s| s.parse::<f64>().ok()).unwrap_or(0.0);

    let mut dist = 0.0;
    if dist_str.contains('\"') {
        // 处理月球地平视差 (Horizontal Parallax) 转换为 AU
        let hp_arcsec = dist_str.replace('\"', "").parse::<f64>().unwrap_or(0.0);
        if hp_arcsec > 0.0 {
            let hp_rad = (hp_arcsec / 3600.0).to_radians();
            // Earth radius / 1 AU in km
            let earth_radius_au = 6378.137 / 149597870.7;
            dist = earth_radius_au / hp_rad.sin();
        }
    } else {
        dist = dist_str.parse::<f64>().unwrap_or(0.0);
    }
    
    Ok(SwetestResult { lon, lat, dist, speed })
}

fn jd_to_year(jd: f64) -> String {
    let years_since_j2000 = (jd - 2451545.0) / 365.25;
    let year = 2000.0 + years_since_j2000;
    if year < 0.0 { format!("{:.0} BC", (1.0 - year).floor()) } else { format!("{:.0} AD", year.floor()) }
}

#[test]
fn test_jpl441_vs_swisseph_detailed_6000_years() {
    let jpl = JplEphemeris::new(JplEphemerisType::DE441).expect("加载 JPL DE441 失败");
    let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    
    // 随机采样 5 个时间点（由于全参数测试输出较多，减少采样点以保持清晰）
    let test_jds: Vec<f64> = (0..5).map(|_| rng.gen_range(1355804.5..=2816912.5)).collect();

    let planets = vec![
        (Planet::Sun, "0", "Sun"),
        (Planet::Moon, "1", "Moon"),
        (Planet::Mars, "4", "Mars"),
    ];

    println!("\n{}", "=".repeat(140));
    println!("JPL DE431 vs Swiss Ephemeris 全参数精度对比 (Lon, Lat, Dist, Speed)");
    println!("{}", "=".repeat(140));

    for jd in test_jds {
        let date_name = jd_to_year(jd);
        println!("\n--- Date: {} (JD {}) ---", date_name, jd);
        println!("{:<10} {:>15} {:>15} {:>15} {:>15}", "Body", "Param", "Swiss Eph", "JPL DE431", "Diff (Arcsec/%)");
        println!("{}", "-".repeat(100));

        for (planet, flag, name) in &planets {
            let swe = run_swetest_detailed(jd, flag).unwrap();
            let pos = jpl.planet_position(*planet, jd).unwrap();

            // 1. Longitude
            let mut d_lon = (swe.lon - pos.longitude_deg()).abs();
            if d_lon > 180.0 { d_lon = 360.0 - d_lon; }
            println!("{:<10} {:>15} {:>15.6} {:>15.6} {:>15.4}\"", name, "Longitude", swe.lon, pos.longitude_deg(), d_lon * 3600.0);

            // 2. Latitude
            let d_lat = (swe.lat - pos.latitude_deg()).abs();
            println!("{:<10} {:>15} {:>15.6} {:>15.6} {:>15.4}\"", "", "Latitude", swe.lat, pos.latitude_deg(), d_lat * 3600.0);

            // 3. Distance (AU)
            let d_dist = (swe.dist - pos.distance_au()).abs();
            let d_dist_pct = (d_dist / swe.dist) * 100.0;
            println!("{:<10} {:>15} {:>15.8} {:>15.8} {:>15.6}%", "", "Distance(AU)", swe.dist, pos.distance_au(), d_dist_pct);

            // 4. Longitude Speed (deg/day)
            let jpl_speed = pos.longitude_speed_deg_day();
            let d_speed = (swe.speed - jpl_speed).abs();
            println!("{:<10} {:>15} {:>15.8} {:>15.8} {:>15.6}\"/day", "", "Lon Speed", swe.speed, jpl_speed, d_speed * 3600.0);
            println!("{}", ".".repeat(100));
        }
    }
}
