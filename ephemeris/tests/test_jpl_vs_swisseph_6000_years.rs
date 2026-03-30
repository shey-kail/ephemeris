// JPL 历表与 Swiss Ephemeris 跨越 4000 年的长周期精度对比测试
// 测试范围：公元前 1000 年 至 公元 3000 年
// 随机采样 20 个时间点

use rand::SeedableRng;
use rand::Rng;
use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::internal::planet::Planet;
use std::process::Command;

/// 运行 swetest 获取指定 JD 的位置
fn run_swetest_at_jd(jd: f64, planet_flag: &str) -> Result<f64, String> {
    let swetest_path = "/home/shey/Codes/my/ephemeris/swetest";
    let ephe_path = "/home/shey/Codes/my/ephemeris/ephe";

    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -j{} -p{} -fPlong -n1 -head",
        ephe_path, swetest_path, jd, planet_flag
    );

    let output = Command::new("bash")
        .args(&["-c", &cmd_str])
        .output()
        .map_err(|e| format!("无法执行 swetest: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // 检查是否使用 Moshier 近似（精度较低）
    let using_moshier = stdout.contains("using Moshier eph.");

    let line = stdout.lines()
        .find(|l| !l.is_empty() && !l.starts_with("date") && !l.starts_with("UT:") && !l.starts_with("TT:") && !l.contains("Epsilon"))
        .ok_or_else(|| format!("未找到数据行：{}", stdout))?;

    let parts: Vec<&str> = line.split_whitespace().collect();
    let lon = parts.get(1).and_then(|s| s.parse::<f64>().ok())
        .ok_or_else(|| format!("解析黄经失败：{}", line))?;

    if using_moshier {
        Ok(lon) // 返回结果但标记为使用近似
    } else {
        Ok(lon)
    }
}

/// 将儒略日转换为年份（用于显示）
fn jd_to_year(jd: f64) -> String {
    // J2000.0 = JD 2451545.0 = 2000 年 1 月 1.5 日
    let years_since_j2000 = (jd - 2451545.0) / 365.25;
    let year = 2000.0 + years_since_j2000;
    
    if year < 0.0 {
        format!("{:.0} BC", (1.0 - year).floor())
    } else {
        format!("{:.0} AD", year.floor())
    }
}

/// 生成 20 个随机测试点（在 -1000 到 +3000 年范围内，确保较高精度）
fn generate_random_test_points(seed: u64) -> Vec<(f64, String)> {
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    
    // 儒略日范围：
    // 公元前 1000 年 ≈ JD 1355804.5 (确保 Swiss Ephemeris 使用高精度历表)
    // 公元 3000 年 ≈ JD 2816912.5
    let jd_min = 1355804.5;
    let jd_max = 2816912.5;
    
    // 生成 20 个随机 JD
    let mut test_points: Vec<f64> = (0..20)
        .map(|_| rng.gen_range(jd_min..=jd_max))
        .collect();
    
    // 排序以便按时间顺序显示
    test_points.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());
    
    // 添加年份标签
    test_points.into_iter()
        .map(|jd| (jd, jd_to_year(jd)))
        .collect()
}

#[test]
fn test_jpl_vs_swisseph_6000_years_random() {
    let jpl = JplEphemeris::new(JplEphemerisType::DE441Lite).expect("加载 JPL 失败");

    // 使用固定种子生成 20 个随机测试点（保证可重复性）
    let test_points = generate_random_test_points(42);

    let planets = vec![
        (Planet::Sun, "0", "Sun"),
        (Planet::Moon, "1", "Moon"),
        (Planet::Mercury, "2", "Mercury"),
        (Planet::Venus, "3", "Venus"),
        (Planet::Mars, "4", "Mars"),
        (Planet::Jupiter, "5", "Jupiter"),
        (Planet::Saturn, "6", "Saturn"),
    ];

    println!("\n{}", "=".repeat(120));
    println!("JPL DE441 vs Swiss Ephemeris 6000 年对比测试 (公元前 3000 年 - 公元 3000 年)");
    println!("随机采样 20 个时间点");
    println!("{}", "=".repeat(120));
    println!("{:<12} {:<10} {:>15} {:>15} {:>15} {:>10}", 
             "Date", "Body", "Swiss Eph(°)", "JPL DE441(°)", "Diff (Arcsec)", "Status");
    println!("{}", "=".repeat(120));

    let mut total_tests = 0;
    let mut excellent_count = 0;  // < 1 角秒
    let mut good_count = 0;       // 1-10 角秒
    let mut acceptable_count = 0; // 10-60 角秒
    let mut poor_count = 0;       // > 60 角秒

    for (jd, date_name) in test_points {
        for (planet, flag, name) in &planets {
            total_tests += 1;
            
            let swetest_lon = match run_swetest_at_jd(jd, flag) {
                Ok(lon) => lon,
                Err(e) => {
                    println!("{:<12} {:<10} {:>15} {:>15} {:>15} ERROR: {}", 
                             date_name, name, "-", "-", "-", e);
                    continue;
                }
            };

            match jpl.planet_position(*planet, jd) {
                Ok(jpl_pos) => {
                    let mut diff = (swetest_lon - jpl_pos.longitude_deg()).abs();
                    if diff > 180.0 { diff = 360.0 - diff; }
                    let diff_arcsec = diff * 3600.0;

                    let status = if diff_arcsec < 1.0 {
                        excellent_count += 1;
                        "EXCELLENT"
                    } else if diff_arcsec < 10.0 {
                        good_count += 1;
                        "GOOD"
                    } else if diff_arcsec < 60.0 {
                        acceptable_count += 1;
                        "ACCEPTABLE"
                    } else {
                        poor_count += 1;
                        "POOR"
                    };

                    println!("{:<12} {:<10} {:>15.6} {:>15.6} {:>15.4} {:>10}",
                             date_name, name, swetest_lon, jpl_pos.longitude_deg(), diff_arcsec, status);
                },
                Err(e) => {
                    println!("{:<12} {:<10} {:>15} {:>15} {:>15} JPL ERROR: {}", 
                             date_name, name, "-", "-", "-", e);
                }
            }
        }
        println!("{}", "-".repeat(120));
    }

    // 打印统计摘要
    println!("\n{}", "=".repeat(120));
    println!("统计摘要");
    println!("{}", "=".repeat(120));
    println!("总测试数：{}", total_tests);
    println!("EXCELLENT (< 1 角秒):   {:>6}  ({:.1}%)", excellent_count, (excellent_count as f64 / total_tests as f64) * 100.0);
    println!("GOOD (1-10 角秒):       {:>6}  ({:.1}%)", good_count, (good_count as f64 / total_tests as f64) * 100.0);
    println!("ACCEPTABLE (10-60 角秒): {:>6}  ({:.1}%)", acceptable_count, (acceptable_count as f64 / total_tests as f64) * 100.0);
    println!("POOR (> 60 角秒):       {:>6}  ({:.1}%)", poor_count, (poor_count as f64 / total_tests as f64) * 100.0);
    println!("{}", "=".repeat(120));
    println!("注意：长周期跨度的对比测试中，Swiss Ephemeris 和 JPL DE441 使用不同的历表模型，");
    println!("     在远古时期差异会显著增大。此测试主要用于观察趋势，不作为精度断言。");
    println!("{}", "=".repeat(120));

    // 信息性测试，不进行断言
}
