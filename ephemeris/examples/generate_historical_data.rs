// 生成历史年份朔望测试数据
// 运行：cargo run --example generate_historical_data --release

use rust_ephemeris::internal::lunnar::so_high;

fn main() {
    println!("=== 历史年份朔望时刻计算 ===\n");
    println!("使用 so_high 函数计算各年份的朔时刻\n");
    
    // 测试数据：年份, w 值（弧度）
    let test_cases: Vec<(i32, f64)> = vec![
        // 古代
        (-3000, -388483.0643576066),
        (-2000, -310772.6284784095),
        (-1000, -233055.9094139052),
        (-500, -194200.6914743067),
        (0, -155345.4735347081),
        // 中世纪
        (500, -116490.2555951095),
        (1000, -77635.0376555110),
        // 近现代
        (2000, 75.3982236862),
        // 未来
        (2500, 38936.8993485919),
        (3000, 77792.1172881905),
    ];
    
    println!("{:<8} {:<20} {:<20} {:<15}", "年份", "w (rad)", "儒略日偏移", "日期");
    println!("{}", "-".repeat(70));
    
    for (year, w) in test_cases {
        let jd_offset = so_high(w);
        let jd_absolute = jd_offset + 2451545.0;
        
        // 转换为公历日期（简化计算）
        let date_str = jd_to_date(jd_absolute);
        
        println!("{:<8} {:<20.6} {:<20.6} {:<15}", year, w, jd_offset, date_str);
    }
}

/// 将儒略日转换为日期字符串（简化版）
fn jd_to_date(jd: f64) -> String {
    // J2000.0 = 2000-01-01 12:00:00
    let j2000_days = jd - 2451545.0;
    
    // 简化计算，只用于显示
    let days = j2000_days.floor() as i64;
    let hours = ((j2000_days - days as f64) * 24.0).floor() as i32;
    let minutes = (((j2000_days - days as f64) * 24.0 - hours as f64) * 60.0).floor() as i32;
    
    // 估算年份（365.2425 天/年）
    let year_approx = 2000 + (days as f64 / 365.2425).floor() as i32;
    
    format!("JD {:.2} (~{}年 {}:{:02})", jd, year_approx, hours, minutes)
}
