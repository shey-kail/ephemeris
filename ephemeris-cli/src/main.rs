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
//! 用法 4（对比模式）- 比较简单模式和精准模式：
//!   ephemeris-calc --compare [选项]
//!
//! 全局选项:
//!   --bsp-path <path>     自定义 JPL BSP 历表文件路径（仅精准模式有效）
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
use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::JulianDate;
use serde::Serialize;
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufWriter, Write};

const RAD_TO_DEG: f64 = 180.0 / std::f64::consts::PI;
const AU_KM: f64 = 149597870.7; // 天文单位（千米）

/// 对比模式结果
#[derive(Serialize)]
struct CompareResult {
    calendar_date: String,
    simple_lon: f64,
    precise_lon: f64,
    simple_lat: f64,
    precise_lat: f64,
    simple_speed: f64,
    precise_speed: f64,
    simple_ra: f64,
    precise_ra: f64,
    simple_dec: f64,
    precise_dec: f64,
}

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
    eprintln!("用法 4（对比模式）- 比较简单模式和精准模式：");
    eprintln!("  ephemeris-calc --compare [选项]");
    eprintln!();
    eprintln!("全局选项:");
    eprintln!("  --bsp-path <path>     自定义 JPL BSP 历表文件路径（仅精准模式有效）");
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
    eprintln!("  # 使用自定义 BSP 历表文件（精准模式）");
    eprintln!("  ephemeris-calc --bsp-path /path/to/custom.bsp 10 2023 7 23 12 0 0");
    eprintln!();
    eprintln!("  # 批处理模式（计算 1000 个日期）");
    eprintln!("  seq 1 1000 | awk '{{print \"10\", 2000, 1, $1, 12, 0, 0}}' | ephemeris-calc --batch");
}

/// 计算单个天体位置并返回 JSON 字符串
fn calculate_single(
    body_id: usize,
    jd_ut: f64,
    tz: f64,
    lon: f64,
    lat: f64,
    input_date_str: &str,
    bsp_path: Option<&str>,
) -> String {
    use rust_ephemeris::astronomy::{calculate_celestial_body, CelestialBody};

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
    
    // 使用简单模式计算（ BSP 路径仅用于精准模式，当前暂时不使用）
    let _ = bsp_path; // 保留参数，暂时不使用
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

        let result = calculate_single(body_id, jd_ut, tz, lon, lat, &input_date_str, None);
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

    // 检查是否启用对比模式
    if args.iter().any(|arg| arg == "--compare") {
        run_compare_mode();
        return;
    }

    // 检查是否启用批处理模式
    if args.iter().any(|arg| arg == "--batch" || arg == "-b") {
        run_batch_mode();
        return;
    }

    // 解析全局参数
    let mut bsp_path: Option<String> = None;
    let mut i = 1;
    while i < args.len() {
        if args[i] == "--bsp-path" && i + 1 < args.len() {
            bsp_path = Some(args[i + 1].clone());
            i += 2;
        } else {
            i += 1;
        }
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

/// 运行对比模式
fn run_compare_mode() {
    let args: Vec<String> = env::args().collect();
    
    // 解析参数
    let mut output_file = "comparison.csv".to_string();
    let mut start_year = 2000;
    let mut start_month = 1;
    let mut start_day = 1;
    let mut start_hour = 12;
    let mut start_minute = 0;
    let mut start_second = 0.0;
    let mut end_year = 2000;
    let mut end_month = 12;
    let mut end_day = 31;
    let mut end_hour = 12;
    let mut end_minute = 0;
    let mut end_second = 0.0;
    let mut step = 1.0; // 步长（天）
    let mut body_id: usize = 10; // 默认月球

    let mut i = 2; // 跳过 "ephemeris-calc" 和 "--compare"
    while i < args.len() {
        match args[i].as_str() {
            "--output" | "-o" => {
                i += 1;
                if i < args.len() {
                    output_file = args[i].clone();
                }
            }
            "--start-year" => {
                i += 1;
                if i < args.len() {
                    start_year = args[i].parse().unwrap_or(2000);
                }
            }
            "--start-month" => {
                i += 1;
                if i < args.len() {
                    start_month = args[i].parse().unwrap_or(1);
                }
            }
            "--start-day" => {
                i += 1;
                if i < args.len() {
                    start_day = args[i].parse().unwrap_or(1);
                }
            }
            "--start-hour" => {
                i += 1;
                if i < args.len() {
                    start_hour = args[i].parse().unwrap_or(12);
                }
            }
            "--start-minute" => {
                i += 1;
                if i < args.len() {
                    start_minute = args[i].parse().unwrap_or(0);
                }
            }
            "--start-second" => {
                i += 1;
                if i < args.len() {
                    start_second = args[i].parse().unwrap_or(0.0);
                }
            }
            "--end-year" => {
                i += 1;
                if i < args.len() {
                    end_year = args[i].parse().unwrap_or(2000);
                }
            }
            "--end-month" => {
                i += 1;
                if i < args.len() {
                    end_month = args[i].parse().unwrap_or(12);
                }
            }
            "--end-day" => {
                i += 1;
                if i < args.len() {
                    end_day = args[i].parse().unwrap_or(31);
                }
            }
            "--end-hour" => {
                i += 1;
                if i < args.len() {
                    end_hour = args[i].parse().unwrap_or(12);
                }
            }
            "--end-minute" => {
                i += 1;
                if i < args.len() {
                    end_minute = args[i].parse().unwrap_or(0);
                }
            }
            "--end-second" => {
                i += 1;
                if i < args.len() {
                    end_second = args[i].parse().unwrap_or(0.0);
                }
            }
            "--step" => {
                i += 1;
                if i < args.len() {
                    step = args[i].parse().unwrap_or(1.0);
                }
            }
            "--body" => {
                i += 1;
                if i < args.len() {
                    body_id = args[i].parse().unwrap_or(10);
                }
            }
            "--help" | "-h" => {
                print_compare_usage();
                return;
            }
            _ => {
                eprintln!("未知参数：{}", args[i]);
                print_compare_usage();
                std::process::exit(1);
            }
        }
        i += 1;
    }

    // 将公历时间转换为儒略日
    let start_jd_obj = JulianDate::from_ymdhms_ut(start_year, start_month, start_day, start_hour, start_minute, start_second);
    let end_jd_obj = JulianDate::from_ymdhms_ut(end_year, end_month, end_day, end_hour, end_minute, end_second);
    let start_jd = start_jd_obj.jd;
    let end_jd = end_jd_obj.jd;
    
    let days = (end_jd - start_jd).abs();

    println!("寿星天文历 - 简单模式 vs 精准模式 对比工具");
    println!("================================================");
    println!("输出文件：{}", output_file);
    println!("起始时间：{:04}-{:02}-{:02} {:02}:{:02}:{:04.1}", start_year, start_month, start_day, start_hour, start_minute, start_second);
    println!("结束时间：{:04}-{:02}-{:02} {:02}:{:02}:{:04.1}", end_year, end_month, end_day, end_hour, end_minute, end_second);
    println!("计算天数：{:.1}", days);
    println!("步长：{} 天", step);
    println!("计算天体：{}", body_name(body_id));
    println!();

    // 创建精准模式 JPL 历表
    let jpl = match JplEphemeris::new(JplEphemerisType::DE441Lite) {
        Ok(ep) => ep,
        Err(e) => {
            eprintln!("加载 JPL 历表失败：{}", e);
            eprintln!("请确保 DE441 BSP 文件存在于 bsp/441/ 目录");
            std::process::exit(1);
        }
    };

    // 创建输出文件
    let file = File::create(&output_file).expect("无法创建输出文件");
    let mut writer = BufWriter::new(file);

    // 写入 CSV 表头
    writeln!(writer, "CalendarDate,SimpleLon,PreciseLon,SimpleLat,PreciseLat,SimpleSpeed,PreciseSpeed,SimpleRA,PreciseRA,SimpleDec,PreciseDec").unwrap();

    let mut total_comparisons = 0;
    let mut excellent_count = 0;
    let mut good_count = 0;
    let mut acceptable_count = 0;
    let mut poor_count = 0;

    let mut jd = start_jd;
    let end_jd = start_jd + days as f64;

    while jd <= end_jd {
        match compare_position(body_id, jd, &jpl) {
            Ok(result) => {
                writeln!(
                    writer,
                    "{},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
                    result.calendar_date,
                    result.simple_lon,
                    result.precise_lon,
                    result.simple_lat,
                    result.precise_lat,
                    result.simple_speed,
                    result.precise_speed,
                    result.simple_ra,
                    result.precise_ra,
                    result.simple_dec,
                    result.precise_dec
                ).unwrap();

                let mut lon_diff = (result.simple_lon - result.precise_lon).abs();
                if lon_diff > 180.0 {
                    lon_diff = 360.0 - lon_diff;
                }
                let lon_diff_arcsec = lon_diff * 3600.0;

                total_comparisons += 1;
                if lon_diff_arcsec < 1.0 {
                    excellent_count += 1;
                } else if lon_diff_arcsec < 10.0 {
                    good_count += 1;
                } else if lon_diff_arcsec < 60.0 {
                    acceptable_count += 1;
                } else {
                    poor_count += 1;
                }
            }
            Err(e) => {
                eprintln!("JD {:.1} 计算错误：{}", jd, e);
            }
        }

        jd += step;
    }

    writer.flush().unwrap();

    println!("\n对比完成！");
    println!("================================================");
    println!("总对比数：{}", total_comparisons);
    println!("EXCELLENT (< 1 角秒):     {:>6} ({:.1}%)", excellent_count, excellent_count as f64 / total_comparisons as f64 * 100.0);
    println!("GOOD (1-10 角秒):         {:>6} ({:.1}%)", good_count, good_count as f64 / total_comparisons as f64 * 100.0);
    println!("ACCEPTABLE (10-60 角秒):  {:>6} ({:.1}%)", acceptable_count, acceptable_count as f64 / total_comparisons as f64 * 100.0);
    println!("POOR (> 60 角秒):         {:>6} ({:.1}%)", poor_count, poor_count as f64 / total_comparisons as f64 * 100.0);
    println!();
    println!("CSV 文件已保存到：{}", output_file);
}

fn body_id_to_celestial(id: usize) -> CelestialBody {
    match id {
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
    }
}

fn compare_position(body_id: usize, jd: f64, jpl: &JplEphemeris) -> Result<CompareResult, String> {
    let calendar_date = jd_to_calendar_date(jd);
    let celestial_body = body_id_to_celestial(body_id);

    let delta_t = rust_ephemeris::internal::math_utils::calc_deltat(jd);
    let jd_tt = jd + delta_t;

    let simple_result = calculate_celestial_body(celestial_body, jd_tt, 0.0, 0.0, 0.0);
    
    let mut simple_lon = simple_result.eclon * RAD_TO_DEG;
    while simple_lon < 0.0 { simple_lon += 360.0; }
    while simple_lon >= 360.0 { simple_lon -= 360.0; }
    
    let simple_lat = simple_result.eclat * RAD_TO_DEG;
    let simple_speed = simple_result.v_eclon * RAD_TO_DEG;
    let simple_ra = simple_result.a_ra * RAD_TO_DEG;
    let simple_dec = simple_result.a_dec * RAD_TO_DEG;

    let (precise_lon, precise_lat, precise_speed, precise_ra, precise_dec) = match body_id {
        10 => {
            let moon_pos = jpl.lunar_position(jd_tt)?;
            let mut lon = moon_pos.longitude_deg();
            while lon < 0.0 { lon += 360.0; }
            while lon >= 360.0 { lon -= 360.0; }
            (lon, moon_pos.latitude.to_degrees(), moon_pos.longitude_speed_deg_day(), moon_pos.apparent_ra_deg(), moon_pos.apparent_dec_deg())
        }
        9 => {
            let sun_pos = jpl.solar_position(jd_tt)?;
            let mut lon = sun_pos.longitude_deg();
            while lon < 0.0 { lon += 360.0; }
            while lon >= 360.0 { lon -= 360.0; }
            (lon, sun_pos.latitude.to_degrees(), sun_pos.longitude_speed_deg_day(), sun_pos.apparent_ra_deg(), sun_pos.apparent_dec_deg())
        }
        _ => {
            use rust_ephemeris::internal::planet::Planet;
            let planet = match body_id {
                1 => Planet::Mercury,
                2 => Planet::Venus,
                3 => Planet::Mars,
                4 => Planet::Jupiter,
                5 => Planet::Saturn,
                _ => return Err("不支持的天体".to_string()),
            };
            let planet_pos = jpl.planet_position(planet, jd_tt)?;
            let mut lon = planet_pos.longitude_deg();
            while lon < 0.0 { lon += 360.0; }
            while lon >= 360.0 { lon -= 360.0; }
            (lon, planet_pos.latitude.to_degrees(), planet_pos.longitude_speed_deg_day(), planet_pos.apparent_ra_deg(), planet_pos.apparent_dec_deg())
        }
    };

    Ok(CompareResult {
        calendar_date,
        simple_lon,
        precise_lon,
        simple_lat,
        precise_lat,
        simple_speed,
        precise_speed,
        simple_ra,
        precise_ra,
        simple_dec,
        precise_dec,
    })
}

fn jd_to_calendar_date(jd: f64) -> String {
    let (year, month, day_float) = JulianDate::jd2day(jd);
    let day = day_float.floor() as i32;
    let hour_float = (day_float - day as f64) * 24.0;
    let hour = hour_float.floor() as i32;
    let minute_float = (hour_float - hour as f64) * 60.0;
    let minute = minute_float.floor() as i32;
    let second = ((minute_float - minute as f64) * 60.0).round() as i32;
    
    format!("{:04}-{:02}-{:02} {:02}:{:02}:{:02}", year, month, day, hour, minute, second)
}

fn print_compare_usage() {
    eprintln!("寿星天文历 - 对比模式");
    eprintln!();
    eprintln!("用法:");
    eprintln!("  ephemeris-calc --compare [选项]");
    eprintln!();
    eprintln!("选项:");
    eprintln!("  --output, -o <file>       输出 CSV 文件路径 (默认：comparison.csv)");
    eprintln!("  --start-year <year>       起始年份 (默认：2000)");
    eprintln!("  --start-month <month>     起始月份 (默认：1)");
    eprintln!("  --start-day <day>         起始日期 (默认：1)");
    eprintln!("  --start-hour <hour>       起始小时 (默认：12)");
    eprintln!("  --start-minute <minute>   起始分钟 (默认：0)");
    eprintln!("  --start-second <second>   起始秒数 (默认：0.0)");
    eprintln!("  --end-year <year>         结束年份 (默认：2000)");
    eprintln!("  --end-month <month>       结束月份 (默认：12)");
    eprintln!("  --end-day <day>           结束日期 (默认：31)");
    eprintln!("  --end-hour <hour>         结束小时 (默认：12)");
    eprintln!("  --end-minute <minute>     结束分钟 (默认：0)");
    eprintln!("  --end-second <second>     结束秒数 (默认：0.0)");
    eprintln!("  --step <days>             步长（天）(默认：1.0)");
    eprintln!("  --body <id>               天体 ID (默认：10 = 月球)");
    eprintln!("                            9=太阳，10=月球，1=水星，2=金星，3=火星，4=木星，5=土星");
    eprintln!("  --help, -h                显示帮助信息");
    eprintln!();
    eprintln!("CSV 格式:");
    eprintln!("  CalendarDate,SimpleLon,PreciseLon,SimpleLat,PreciseLat,SimpleSpeed,PreciseSpeed,SimpleRA,PreciseRA,SimpleDec,PreciseDec");
    eprintln!();
    eprintln!("示例:");
    eprintln!("  # 对比月球在 2000 年全年的位置（默认步长 1 天）");
    eprintln!("  ephemeris-calc --compare --start-year 2000 --start-month 1 --start-day 1 --end-year 2000 --end-month 12 --end-day 31");
    eprintln!();
    eprintln!("  # 对比太阳在 2000 年的位置（步长 10 天）");
    eprintln!("  ephemeris-calc --compare --body 9 --step 10 --start-year 2000 --end-year 2000");
    eprintln!();
    eprintln!("  # 对比远古时期月球（公元前 1000 年全年）");
    eprintln!("  ephemeris-calc --compare --start-year -1000 --end-year -1000 --body 10");
    eprintln!();
    eprintln!("  # 对比月球在指定月份（步长 0.5 天 = 12 小时）");
    eprintln!("  ephemeris-calc --compare --start-year 2024 --start-month 1 --end-month 1 --step 0.5");
}
