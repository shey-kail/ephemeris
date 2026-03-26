//! 寿星天文历 CLI 工具
//!
//! 用法 1（推荐）- 使用格里高利历时间：
//!   ephemeris-calc <body_id> <year> <month> <day> <hour> <minute> <second> [tz] [lon] [lat]
//!
//! 用法 2（兼容旧版）- 使用儒略日：
//!   ephemeris-calc --jd <body_id> <jd> [tz] [lon] [lat]
//!
//! 用法 3（批处理模式）- 从 stdin 读取多个查询：
//!   echo "10 2000 1 1 12 0 0" | ephemeris-calc --batch
//!   或：ephemeris-calc --batch < queries.txt
//!   每行格式：body_id year month day hour minute second [tz] [lon] [lat]
//!
//! 参数:
//!   body_id: 天体 ID (0:地球，1:水星，... 9:太阳，10:月亮)
//!   year: 年 (例如 2023)
//!   month: 月 (1-12)
//!   day: 日 (1-31)
//!   hour: 时 (0-23)
//!   minute: 分 (0-59)
//!   second: 秒 (0.0-60.0)
//!   jd: 绝对儒略日 (例如 2451545.0)
//!   tz: 时区 (默认 0)
//!   lon: 经度，弧度制 (默认 0)
//!   lat: 纬度，弧度制 (默认 0)

use rust_ephemeris::astronomy::{calculate_celestial_body, CelestialBody};
use rust_ephemeris::JulianDate;
use serde::Serialize;
use std::env;
use std::io::{self, BufRead, Write};

const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
const AU_KM: f64 = 149597870.7; // 天文单位（千米）

#[derive(Serialize)]
struct EphemerisResult {
    body_name: String,
    jd_tt: f64,
    jd_ut: f64,
    input_date: String,
    delta_t: f64,
    eclon: f64,
    eclat: f64,
    a_lon: f64,
    a_lat: f64,
    a_ra: f64,
    a_dec: f64,
    r: f64,
    d_e: f64,
    lt: f64,
    st_ra: f64,
    st_dec: f64,
    dist: f64,
    az: f64,
    alt: f64,
    v_eclon: f64,
    v_d_e: f64,
}

fn body_name(id: usize) -> &'static str {
    match id {
        0 => "Earth",
        1 => "Mercury",
        2 => "Venus",
        3 => "Mars",
        4 => "Jupiter",
        5 => "Saturn",
        6 => "Uranus",
        7 => "Neptune",
        8 => "Pluto",
        9 => "Sun",
        10 => "Moon",
        _ => "Unknown",
    }
}

fn print_usage() {
    eprintln!("寿星天文历 CLI 工具");
    eprintln!();
    eprintln!("用法 1（推荐）- 使用格里高利历时间：");
    eprintln!("  ephemeris-calc <body_id> <year> <month> <day> <hour> <minute> <second> [tz] [lon] [lat]");
    eprintln!();
    eprintln!("用法 2（兼容旧版）- 使用儒略日：");
    eprintln!("  ephemeris-calc --jd <body_id> <jd> [tz] [lon] [lat]");
    eprintln!();
    eprintln!("用法 3（批处理模式）- 从 stdin 读取多个查询：");
    eprintln!("  echo \"10 2000 1 1 12 0 0\" | ephemeris-calc --batch");
    eprintln!("  或：ephemeris-calc --batch < queries.txt");
    eprintln!("  每行格式：body_id year month day hour minute second [tz] [lon] [lat]");
    eprintln!();
    eprintln!("参数:");
    eprintln!("  body_id: 天体 ID (0:地球，1:水星，2:金星，3:火星，4:木星，5:土星，6:天王星，7:海王星，8:冥王星，9:太阳，10:月亮)");
    eprintln!("  year: 年 (例如 2023)");
    eprintln!("  month: 月 (1-12)");
    eprintln!("  day: 日 (1-31)");
    eprintln!("  hour: 时 (0-23)");
    eprintln!("  minute: 分 (0-59)");
    eprintln!("  second: 秒 (0.0-60.0)");
    eprintln!("  jd: 绝对儒略日 (例如 2451545.0)");
    eprintln!("  tz: 时区 (默认 0)，东八区为 -8");
    eprintln!("  lon: 经度，度数制 (默认 0)，东经为正");
    eprintln!("  lat: 纬度，度数制 (默认 0)，北纬为正");
    eprintln!();
    eprintln!("示例:");
    eprintln!("  # 计算 2023 年 7 月 23 日 12:00:00 水星位置（东八区，北京）");
    eprintln!("  ephemeris-calc 1 2023 7 23 12 0 0 -8 116.383 39.9");
    eprintln!();
    eprintln!("  # 使用儒略日（兼容旧版）");
    eprintln!("  ephemeris-calc --jd 1 2460149.0 -8 116.383 39.9");
    eprintln!();
    eprintln!("  # 批处理模式（计算 1000 个日期）");
    eprintln!("  seq 1 1000 | awk '{{print \"10\", 2000, 1, $1, 12, 0, 0}}' | ephemeris-calc --batch");
}

/// 计算单个天体位置并返回 JSON 字符串
fn calculate_single(body_id: usize, jd_ut: f64, tz: f64, lon: f64, lat: f64, input_date_str: &str) -> String {
    let body = match body_id {
        0 => CelestialBody::Earth,
        1 => CelestialBody::Mercury,
        2 => CelestialBody::Venus,
        3 => CelestialBody::Mars,
        4 => CelestialBody::Jupiter,
        5 => CelestialBody::Saturn,
        6 => CelestialBody::Uranus,
        7 => CelestialBody::Neptune,
        8 => CelestialBody::Pluto,
        9 => CelestialBody::Sun,
        10 => CelestialBody::Moon,
        _ => CelestialBody::Sun,
    };

    let jd_tt = jd_ut + rust_ephemeris::internal::math_utils::calc_deltat(jd_ut);
    let result = calculate_celestial_body(body, jd_tt, tz, lon, lat);

    let output = EphemerisResult {
        body_name: body_name(body_id).to_string(),
        jd_tt,
        jd_ut,
        input_date: input_date_str.to_string(),
        delta_t: rust_ephemeris::internal::math_utils::calc_deltat(jd_ut),
        eclon: result.eclon * RAD_TO_DEG,
        eclat: result.eclat * RAD_TO_DEG,
        a_lon: result.a_lon * RAD_TO_DEG,
        a_lat: result.a_lat * RAD_TO_DEG,
        a_ra: result.a_ra * RAD_TO_DEG,
        a_dec: result.a_dec * RAD_TO_DEG,
        r: result.r,
        d_e: if body_id == 10 { result.d_e / AU_KM } else { result.d_e },
        lt: if body_id == 10 { result.lt / AU_KM } else { result.lt },
        st_ra: result.st_ra * RAD_TO_DEG,
        st_dec: result.st_dec * RAD_TO_DEG,
        dist: if body_id == 10 { result.dist / AU_KM } else { result.dist },
        az: result.az * RAD_TO_DEG,
        alt: result.alt * RAD_TO_DEG,
        v_eclon: result.v_eclon * RAD_TO_DEG,
        v_d_e: if body_id == 10 { result.v_d_e / AU_KM } else { result.v_d_e },
    };

    serde_json::to_string(&output).unwrap()
}

/// 批处理模式：从 stdin 读取多个查询
fn run_batch_mode() {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut stdout_lock = stdout.lock();
    
    let mut count = 0;
    
    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue; // 跳过空行和注释
        }
        
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 7 {
            eprintln!("警告：跳过无效行（需要至少 7 个参数）: {}", line);
            continue;
        }
        
        let body_id: usize = parts[0].parse().unwrap_or(9);
        let year: i32 = parts[1].parse().unwrap_or(2000);
        let month: i32 = parts[2].parse().unwrap_or(1);
        let day: i32 = parts[3].parse().unwrap_or(1);
        let hour: i32 = parts[4].parse().unwrap_or(0);
        let minute: i32 = parts[5].parse().unwrap_or(0);
        let second: f64 = parts[6].parse().unwrap_or(0.0);
        
        let tz: f64 = parts.get(7).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let lon_deg: f64 = parts.get(8).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let lat_deg: f64 = parts.get(9).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        
        let lon = lon_deg * std::f64::consts::PI / 180.0;
        let lat = lat_deg * std::f64::consts::PI / 180.0;
        
        let jd_ut_obj = JulianDate::from_ymdhms_ut(year, month, day, hour, minute, second);
        let jd_ut = jd_ut_obj.jd;
        
        let input_date_str = format!("{:04}-{:02}-{:02} {:02}:{:02}:{:.1}", year, month, day, hour, minute, second);
        
        let result = calculate_single(body_id, jd_ut, tz, lon, lat, &input_date_str);
        writeln!(stdout_lock, "{}", result).unwrap();
        
        count += 1;
    }
    
    eprintln!("批处理完成：处理了 {} 个查询", count);
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        std::process::exit(1);
    }

    // 检查是否启用批处理模式
    if args.iter().any(|arg| arg == "--batch" || arg == "-b") {
        run_batch_mode();
        return;
    }

    let use_jd_mode = args.iter().any(|arg| arg == "--jd");

    let (body_id, jd_tt, jd_ut, dt, input_date_str) = if use_jd_mode {
        // 儒略日模式（兼容旧版）
        if args.len() < 4 {
            eprintln!("错误：儒略日模式需要至少 3 个参数");
            print_usage();
            std::process::exit(1);
        }
        let body_id: usize = args[2].parse().unwrap_or(9);
        let jd: f64 = args[3].parse().unwrap_or(0.0);
        let input_date_str = format!("JD {:.1}", jd);
        // 儒略日模式下，假设输入的是 UT1 时间
        let jd_ut = jd;
        let dt = rust_ephemeris::internal::math_utils::calc_deltat(jd_ut);
        let jd_tt = jd_ut + dt;
        (body_id, jd_tt, jd_ut, dt, input_date_str)
    } else {
        // 格里高利历模式（新版推荐）
        if args.len() < 8 {
            eprintln!("错误：格里高利历模式需要至少 7 个参数");
            print_usage();
            std::process::exit(1);
        }
        let body_id: usize = args[1].parse().unwrap_or(9);
        let year: i32 = args[2].parse().unwrap_or(2000);
        let month: i32 = args[3].parse().unwrap_or(1);
        let day: i32 = args[4].parse().unwrap_or(1);
        let hour: i32 = args[5].parse().unwrap_or(0);
        let minute: i32 = args[6].parse().unwrap_or(0);
        let second: f64 = args[7].parse().unwrap_or(0.0);

        // 先计算 UT1 儒略日
        let jd_ut_obj = JulianDate::from_ymdhms_ut(year, month, day, hour, minute, second);
        let jd_ut = jd_ut_obj.jd;

        // 计算 ΔT 和 TT（使用新的 calc_deltat 函数）
        let dt = rust_ephemeris::internal::math_utils::calc_deltat(jd_ut);
        let jd_tt = jd_ut + dt;
        
        let input_date_str = format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:.1}",
            year, month, day, hour, minute, second
        );
        (body_id, jd_tt, jd_ut, dt, input_date_str)
    };

    let tz: f64 = args
        .get(if use_jd_mode { 4 } else { 8 })
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let lon_deg: f64 = args
        .get(if use_jd_mode { 5 } else { 9 })
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);
    let lat_deg: f64 = args
        .get(if use_jd_mode { 6 } else { 10 })
        .and_then(|s| s.parse().ok())
        .unwrap_or(0.0);

    // 度数转弧度
    let lon = lon_deg * std::f64::consts::PI / 180.0;
    let lat = lat_deg * std::f64::consts::PI / 180.0;

    // 获取天体
    let body = match body_id {
        0 => CelestialBody::Earth,
        1 => CelestialBody::Mercury,
        2 => CelestialBody::Venus,
        3 => CelestialBody::Mars,
        4 => CelestialBody::Jupiter,
        5 => CelestialBody::Saturn,
        6 => CelestialBody::Uranus,
        7 => CelestialBody::Neptune,
        8 => CelestialBody::Pluto,
        9 => CelestialBody::Sun,
        10 => CelestialBody::Moon,
        _ => CelestialBody::Sun,
    };

    // 计算天体位置（使用 TT）
    let result = calculate_celestial_body(body, jd_tt, tz, lon, lat);

    // 输出 JSON（到 stdout）- 将弧度转换为角度
    let output = EphemerisResult {
        body_name: body_name(body_id).to_string(),
        jd_tt,
        jd_ut,
        input_date: input_date_str,
        delta_t: dt,
        eclon: result.eclon * RAD_TO_DEG,
        eclat: result.eclat * RAD_TO_DEG,
        a_lon: result.a_lon * RAD_TO_DEG,
        a_lat: result.a_lat * RAD_TO_DEG,
        a_ra: result.a_ra * RAD_TO_DEG,
        a_dec: result.a_dec * RAD_TO_DEG,
        r: result.r,
        d_e: if body_id == 10 { result.d_e / AU_KM } else { result.d_e },
        lt: if body_id == 10 { result.lt / AU_KM } else { result.lt },
        st_ra: result.st_ra * RAD_TO_DEG,
        st_dec: result.st_dec * RAD_TO_DEG,
        dist: if body_id == 10 { result.dist / AU_KM } else { result.dist },
        az: result.az * RAD_TO_DEG,
        alt: result.alt * RAD_TO_DEG,
        v_eclon: result.v_eclon * RAD_TO_DEG,
        v_d_e: if body_id == 10 { result.v_d_e / AU_KM } else { result.v_d_e },
    };

    println!("{}", serde_json::to_string(&output).unwrap());
}
