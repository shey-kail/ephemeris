// IAU 2000A 章动模型实现
// 基于 swisseph/swenut2000a.h 和 swisseph/swephlib.c 中的 calc_nutation_iau2000ab

use crate::internal::math_utils;

/// 将度归一化到 0-360 度
fn deg2mrad(deg: f64) -> f64 {
    use std::f64::consts::PI;
    let r = deg - (deg / 360.0).floor() * 360.0;
    let r = if r < 0.0 { r + 360.0 } else { r };
    r * PI / 180.0
}

/// 计算 IAU 2000A 章动
/// 
/// # Arguments
/// * `jd` - 儒略日 (TT)
/// 
/// # Returns
/// * `(nutation_lon, nutation_lat)` - 黄经章动和黄赤交角章动 (弧度)
pub fn nutation_iau2000a(jd: f64) -> (f64, f64) {
    use super::nutation_iau2000a::*;
    
    const J2000: f64 = 2451545.0;
    const DEGTORAD: f64 = std::f64::consts::PI / 180.0;
    
    let t = (jd - J2000) / 36525.0;
    
    // 日月章动的基本参数 (Simon & al. 1994)
    // 平黄经参数，单位：度，转为弧度
    let m = deg2mrad((485868.249036 
        + t * (1717915923.2178 
        + t * (31.8792 
        + t * (0.051635 
        + t * (-0.00024470))))) / 3600.0);
    
    let sm = deg2mrad((1287104.79305 
        + t * (129596581.0481 
        + t * (-0.5532 
        + t * (0.000136 
        + t * (-0.00001149))))) / 3600.0);
    
    let f = deg2mrad((335779.526232 
        + t * (1739527262.8478 
        + t * (-12.7512 
        + t * (-0.001037 
        + t * (0.00000417))))) / 3600.0);
    
    let d = deg2mrad((1072260.70369 
        + t * (1602961601.2090 
        + t * (-6.3706 
        + t * (0.006593 
        + t * (-0.00003169))))) / 3600.0);
    
    let om = deg2mrad((450160.398036 
        + t * (-6962890.5431 
        + t * (7.4722 
        + t * (0.007702 
        + t * (-0.00005939))))) / 3600.0);
    
    // 日月章动级数
    let mut dpsi = 0.0_f64;
    let mut deps = 0.0_f64;
    
    for i in (0..NLS).rev() {
        let j = i * 5;
        let k = i * 6;
        
        let darg = math_utils::rad2mrad(
            NLS_COEFFS[j + 0] as f64 * m +
            NLS_COEFFS[j + 1] as f64 * sm +
            NLS_COEFFS[j + 2] as f64 * f +
            NLS_COEFFS[j + 3] as f64 * d +
            NLS_COEFFS[j + 4] as f64 * om
        );
        
        let sinarg = darg.sin();
        let cosarg = darg.cos();
        
        dpsi += (CLS_COEFFS[k + 0] as f64 + CLS_COEFFS[k + 1] as f64 * t) * sinarg 
              + CLS_COEFFS[k + 2] as f64 * cosarg;
        deps += (CLS_COEFFS[k + 3] as f64 + CLS_COEFFS[k + 4] as f64 * t) * cosarg 
              + CLS_COEFFS[k + 5] as f64 * sinarg;
    }
    
    let mut nut_lon = dpsi * O1MAS2DEG;
    let mut nut_lat = deps * O1MAS2DEG;
    
    // 行星章动 (IAU 2000A)
    // 基本参数 (MHB2000)
    let al = math_utils::rad2mrad(2.35555598 + 8328.6914269554 * t);
    let alsu = math_utils::rad2mrad(6.24006013 + 628.301955 * t);
    let af = math_utils::rad2mrad(1.627905234 + 8433.466158131 * t);
    let ad = math_utils::rad2mrad(5.198466741 + 7771.3771468121 * t);
    let aom = math_utils::rad2mrad(2.18243920 - 33.757045 * t);
    
    // 行星平黄经 (Souchay et al. 1999)
    let alme = math_utils::rad2mrad(4.402608842 + 2608.7903141574 * t);
    let alve = math_utils::rad2mrad(3.176146697 + 1021.3285546211 * t);
    let alea = math_utils::rad2mrad(1.753470314 + 628.3075849991 * t);
    let alma = math_utils::rad2mrad(6.203480913 + 334.0612426700 * t);
    let alju = math_utils::rad2mrad(0.599546497 + 52.9690962641 * t);
    let alsa = math_utils::rad2mrad(0.874016757 + 21.3299104960 * t);
    let alur = math_utils::rad2mrad(5.481293871 + 7.4781598567 * t);
    let alne = math_utils::rad2mrad(5.321159000 + 3.8127774000 * t);
    
    // 岁差
    let apa = (0.02438175 + 0.00000538691 * t) * t;
    
    // 行星章动级数
    dpsi = 0.0;
    deps = 0.0;
    
    for i in (0..NPL).rev() {
        let j = i * 14;
        let k = i * 4;
        
        let darg = math_utils::rad2mrad(
            NPL_COEFFS[j + 0] as f64 * al +
            NPL_COEFFS[j + 1] as f64 * alsu +
            NPL_COEFFS[j + 2] as f64 * af +
            NPL_COEFFS[j + 3] as f64 * ad +
            NPL_COEFFS[j + 4] as f64 * aom +
            NPL_COEFFS[j + 5] as f64 * alme +
            NPL_COEFFS[j + 6] as f64 * alve +
            NPL_COEFFS[j + 7] as f64 * alea +
            NPL_COEFFS[j + 8] as f64 * alma +
            NPL_COEFFS[j + 9] as f64 * alju +
            NPL_COEFFS[j + 10] as f64 * alsa +
            NPL_COEFFS[j + 11] as f64 * alur +
            NPL_COEFFS[j + 12] as f64 * alne +
            NPL_COEFFS[j + 13] as f64 * apa
        );
        
        let sinarg = darg.sin();
        let cosarg = darg.cos();
        
        dpsi += ICPL_COEFFS[k + 0] as f64 * sinarg + ICPL_COEFFS[k + 1] as f64 * cosarg;
        deps += ICPL_COEFFS[k + 2] as f64 * sinarg + ICPL_COEFFS[k + 3] as f64 * cosarg;
    }
    
    nut_lon += dpsi * O1MAS2DEG;
    nut_lat += deps * O1MAS2DEG;
    
    // IAU 2006 P03 岁差修正 (Capitaine et al. A & A 412, 366 (2005))
    let dpsi = -8.1 * om.sin() 
             - 0.6 * (2.0 * f - 2.0 * d + 2.0 * om).sin()
             + t * (47.8 * om.sin() 
                  + 3.7 * (2.0 * f - 2.0 * d + 2.0 * om).sin() 
                  + 0.6 * (2.0 * f + 2.0 * om).sin() 
                  - 0.6 * (2.0 * om).sin());
    
    let deps = t * (-25.6 * om.cos() - 1.6 * (2.0 * f - 2.0 * d + 2.0 * om).cos());
    
    nut_lon += dpsi / (3600.0 * 1000000.0);
    nut_lat += deps / (3600.0 * 1000000.0);
    
    // 转换为弧度
    (nut_lon * DEGTORAD, nut_lat * DEGTORAD)
}

/// 简化版 IAU 2000B 章动 (仅日月章动，77 项)
pub fn nutation_iau2000b(jd: f64) -> (f64, f64) {
    use super::nutation_iau2000a::*;

    const J2000: f64 = 2451545.0;
    const DEGTORAD: f64 = std::f64::consts::PI / 180.0;

    let t = (jd - J2000) / 36525.0;

    // 日月章动的基本参数
    let m = deg2mrad((485868.249036
        + t * (1717915923.2178
        + t * (31.8792
        + t * (0.051635
        + t * (-0.00024470))))) / 3600.0);

    let sm = deg2mrad((1287104.79305
        + t * (129596581.0481
        + t * (-0.5532
        + t * (0.000136
        + t * (-0.00001149))))) / 3600.0);

    let f = deg2mrad((335779.526232
        + t * (1739527262.8478
        + t * (-12.7512
        + t * (-0.001037
        + t * (0.00000417))))) / 3600.0);

    let d = deg2mrad((1072260.70369
        + t * (1602961601.2090
        + t * (-6.3706
        + t * (0.006593
        + t * (-0.00003169))))) / 3600.0);

    let om = deg2mrad((450160.398036
        + t * (-6962890.5431
        + t * (7.4722
        + t * (0.007702
        + t * (-0.00005939))))) / 3600.0);
    
    // 日月章动级数 (仅 77 项)
    let mut dpsi = 0.0_f64;
    let mut deps = 0.0_f64;
    
    for i in (0..NLS_2000B).rev() {
        let j = i * 5;
        let k = i * 6;
        
        let darg = math_utils::rad2mrad(
            NLS_COEFFS[j + 0] as f64 * m +
            NLS_COEFFS[j + 1] as f64 * sm +
            NLS_COEFFS[j + 2] as f64 * f +
            NLS_COEFFS[j + 3] as f64 * d +
            NLS_COEFFS[j + 4] as f64 * om
        );
        
        let sinarg = darg.sin();
        let cosarg = darg.cos();
        
        dpsi += (CLS_COEFFS[k + 0] as f64 + CLS_COEFFS[k + 1] as f64 * t) * sinarg 
              + CLS_COEFFS[k + 2] as f64 * cosarg;
        deps += (CLS_COEFFS[k + 3] as f64 + CLS_COEFFS[k + 4] as f64 * t) * cosarg 
              + CLS_COEFFS[k + 5] as f64 * sinarg;
    }
    
    let nut_lon = dpsi * O1MAS2DEG;
    let nut_lat = deps * O1MAS2DEG;
    
    (nut_lon * DEGTORAD, nut_lat * DEGTORAD)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nutation_iau2000a() {
        // 测试 J2000 时刻
        let (lon, lat) = nutation_iau2000a(2451545.0);
        println!("J2000: lon={:.6}°, lat={:.6}°", lon * 180.0 / std::f64::consts::PI * 3600.0, lat * 180.0 / std::f64::consts::PI * 3600.0);
        
        // 测试 2026-01-01
        let jd = 2461041.5008;
        let (lon, lat) = nutation_iau2000a(jd);
        println!("2026-01-01: lon={:.6}°, lat={:.6}°", lon * 180.0 / std::f64::consts::PI * 3600.0, lat * 180.0 / std::f64::consts::PI * 3600.0);
    }
}
