// 调试岁差计算

use rust_ephemeris::internal::jpl_ephemeris::{JplEphemeris, JplEphemerisType};
use rust_ephemeris::internal::corrections::{precession_vondrak_2011, apply_precession_iau2006};
use rust_ephemeris::internal::math_utils::xyz2llr;

#[test]
fn test_debug_precession_detail() {
    let jpl = JplEphemeris::new(JplEphemerisType::DE441Lite).expect("加载 JPL 失败");
    
    println!("\n{}", "=".repeat(100));
    println!("岁差计算详细调试");
    println!("{}", "=".repeat(100));
    
    let test_cases = vec![
        (2451545.0, "J2000.0"),
        (2415020.5, "1900-01-01"),
        (2433261.230743533, "1949"),
        (1355866.5, "公元前 1000 年"),
    ];
    
    for (jd, desc) in test_cases {
        println!("\n{} (JD {}):", desc, jd);
        
        // 测试 Vondrák 2011 矩阵
        let matrix = precession_vondrak_2011(jd);
        println!("  Vondrák 2011 矩阵:");
        println!("    [{:.10}, {:.10}, {:.10}]", matrix[0][0], matrix[0][1], matrix[0][2]);
        println!("    [{:.10}, {:.10}, {:.10}]", matrix[1][0], matrix[1][1], matrix[1][2]);
        println!("    [{:.10}, {:.10}, {:.10}]", matrix[2][0], matrix[2][1], matrix[2][2]);
        
        // 测试单位向量变换
        let x_axis = (1.0, 0.0, 0.0);
        let transformed = (
            x_axis.0 * matrix[0][0] + x_axis.1 * matrix[0][1] + x_axis.2 * matrix[0][2],
            x_axis.0 * matrix[1][0] + x_axis.1 * matrix[1][1] + x_axis.2 * matrix[1][2],
            x_axis.0 * matrix[2][0] + x_axis.1 * matrix[2][1] + x_axis.2 * matrix[2][2],
        );
        let (lon, lat, _) = xyz2llr(transformed);
        println!("  X 轴变换后：黄经={:.6}°, 黄纬={:.6}°", lon.to_degrees(), lat.to_degrees());
        
        match jpl.solar_position(jd) {
            Ok(pos) => {
                println!("  JPL 黄经：{:.10}°", pos.longitude_deg());
                println!("  JPL 黄纬：{:.10}°", pos.latitude_deg());
                println!("  距离：{:.6} AU", pos.distance);
            },
            Err(e) => println!("  JPL 错误：{}", e),
        }
    }
    
    println!("\n{}", "=".repeat(100));
}
