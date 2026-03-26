#[allow(dead_code)]
use std::f64::consts::PI;
use crate::internal::constants;

/// 角度
///
/// 主要处理角度归化，角度转换等功能
#[derive(Default, Debug)]
pub struct Angle {
    /// 弧度\$ 0-2\pi $
    pub rad: f64,
    /// 弧度\$ -\pi-\pi $
    pub mrad: f64,

    pub hours: Option<i32>,
    pub minutes: Option<i32>,
    pub seconds: Option<f64>,

    pub deg: Option<i32>,
    pub deg_m: Option<i32>,
    pub deg_s: Option<f64>,
}

impl Angle {
    /// 从浮点数构造度数
    pub fn from_f64(x: f64) -> Self {
        let mut a = x - (x / (PI * 2.0)).floor() * (PI * 2.0);
        if a < 0.0 {
            a = a + 2.0 * PI;
        }
        let mut b = a;
        if b > PI {
            b = b - 2.0 * PI;
        }
        Angle {
            rad: a,
            mrad: b,
            ..Default::default()
        }
    }

    /// 从角度构造度数
    pub fn from_degress(deg: &str) -> Self {
        let parts: Vec<&str> = deg.split(|c| c == '°' || c == '\'' || c == '\"').collect();
        let degress = parts[0].trim().parse::<i32>().unwrap();
        let minutes = parts[1].trim().parse::<i32>().unwrap();
        let seconds = parts[2].trim().parse::<f64>().unwrap();
        let f: f64 = (degress as f64) + (minutes as f64) / 60.0 + seconds / 60.0 / 60.0;
        let mut angle = Angle::from_f64((f * PI) / 180.0);
        angle.deg = Some(degress);
        angle.deg_m = Some(minutes);
        angle.deg_s = Some(seconds);
        angle
    }

    pub fn f2tuple(mut cur: f64, ext: i32) -> (i32, i32, f64) {
        let base = 10.0_f64;
        let mut d = cur.floor() as i32;
        cur = (cur - (d as f64)) * 60.0;
        let mut m = cur.floor() as i32;
        cur = (cur - (m as f64)) * 60.0;
        let mut s = cur * base.powi(ext);
        s = s.round();
        s /= base.powi(ext);
        if s > 60.0 {
            s -= 1.0;
            m += 1;
        }
        if m > 60 {
            m -= 1;
            d += 1;
        }
        (d, m, s)
    }


    /// 弧度制输出
    pub fn radis(&self) -> String {
        format!("radis:{:?}, mradis:{:?}", self.rad, self.mrad)
    }

    /// 时间制输出
    pub fn time(&mut self, ext: i32) -> String {
        match (self.hours, self.minutes, self.seconds) {
            (Some(x), Some(y), Some(z)) => { format!("{:?}h {:?}m {:?}s", x, y, z) }
            _ => {
                let cur = (self.rad / PI) * 12.0;
                let (d, m, s) = Self::f2tuple(cur, ext);
                (self.hours, self.minutes, self.seconds) = (Some(d), Some(m), Some(s));

                format!("{:?}h {:?}m {:?}s", d, m, s)
            }
        }
    }

    /// 输出度数
    pub fn degress(&mut self, ext: i32) -> String {
        match (self.deg, self.deg_m, self.deg_s) {
            (Some(x), Some(y), Some(z)) => { format!("{:?}° {:?}' {:?}\"", x, y, z) }
            _ => {
                let cur = (self.rad / PI) * 180.0;
                let (d, m, s) = Self::f2tuple(cur, ext);
                (self.deg, self.deg_m, self.deg_s) = (Some(d), Some(m), Some(s));
                format!("{:?}° {:?}' {:?}\"", d, m, s)
            }
        }
    }
}

#[test]
fn test_angle() {
    print!("to angle {:?}\n", Angle::from_f64(-11.24));
    let a = Angle::from_f64(-11.24).radis();
    println!("{}", &a);

    let mut c = Angle::from_f64(PI / 6.0);
    println!("{}", c.degress(2));

    let mut e = Angle::from_degress("30° 0' 0.0\"");

    println!("{}", e.radis());
    println!("{}", e.time(2));
}

// 计算 TD-UT 相关的计算
// 完整移植 swetest 的 calc_deltat 逻辑

const J2000: f64 = 2451545.0;

/// 潮汐加速度参数
const SE_TIDAL_STEPHENSON_2016: f64 = -25.85;
const SE_TIDAL_DE431: f64 = -25.80;  // swetest 默认 tid_acc
const SE_TIDAL_ANCIENT: f64 = -25.95;  // 古代年份（公元前 720 年以前）使用的参数
const SE_TIDAL_26: f64 = -26.0;

/// 调整潮汐加速度差异
fn adjust_for_tidacc(ans: f64, y: f64, tid_acc: f64, tid_acc0: f64, adjust_after_1955: bool) -> f64 {
    if y < 1955.0 || adjust_after_1955 {
        let b = y - 1955.0;
        ans + (-0.000091 * (tid_acc - tid_acc0) * b * b)
    } else {
        ans
    }
}

/// Stephenson et al. (2016) 样条插值表
/// 每行包含：[jd_start, jd_end, c0, c1, c2, c3]
const DTCF16: [[f64; 6]; 54] = [
    [1458085.5, 1867156.5, 20550.593, -21268.478, 11863.418, -4541.129],
    [1867156.5, 2086302.5, 6604.404, -5981.266, -505.093, 1349.609],
    [2086302.5, 2268923.5, 1467.654, -2452.187, 2460.927, -1183.759],
    [2268923.5, 2305447.5, 292.635, -216.322, -43.614, 56.681],
    [2305447.5, 2323710.5, 89.380, -66.754, 31.607, -10.497],
    [2323710.5, 2349276.5, 43.736, -49.043, 0.227, 15.811],
    [2349276.5, 2378496.5, 10.730, -1.321, 62.250, -52.946],
    [2378496.5, 2382148.5, 18.714, -4.457, -1.509, 2.507],
    [2382148.5, 2385800.5, 15.255, 0.046, 6.012, -4.634],
    [2385800.5, 2389453.5, 16.679, -1.831, -7.889, 3.799],
    [2389453.5, 2393105.5, 10.758, -6.211, 3.509, -0.388],
    [2393105.5, 2396758.5, 7.668, -0.357, 2.345, -0.338],
    [2396758.5, 2398584.5, 9.317, 1.659, 0.332, -0.932],
    [2398584.5, 2400410.5, 10.376, -0.472, -2.463, 1.596],
    [2400410.5, 2402237.5, 9.038, -0.610, 2.325, -2.497],
    [2402237.5, 2404063.5, 8.256, -3.450, -5.166, 2.729],
    [2404063.5, 2405889.5, 2.369, -5.596, 3.020, -0.919],
    [2405889.5, 2407715.5, -1.126, -2.312, 0.264, -0.037],
    [2407715.5, 2409542.5, -3.211, -1.894, 0.154, 0.562],
    [2409542.5, 2411368.5, -4.388, 0.101, 1.841, -1.438],
    [2411368.5, 2413194.5, -3.884, -0.531, -2.473, 1.870],
    [2413194.5, 2415020.5, -5.017, 0.134, 3.138, -0.232],
    [2415020.5, 2416846.5, -1.977, 5.715, 2.443, -1.257],
    [2416846.5, 2418672.5, 4.923, 6.828, -1.329, 0.720],
    [2418672.5, 2420498.5, 11.142, 6.330, 0.831, -0.825],
    [2420498.5, 2422324.5, 17.479, 5.518, -1.643, 0.262],
    [2422324.5, 2424151.5, 21.617, 3.020, -0.856, 0.008],
    [2424151.5, 2425977.5, 23.789, 1.333, -0.831, 0.127],
    [2425977.5, 2427803.5, 24.418, 0.052, -0.449, 0.142],
    [2427803.5, 2429629.5, 24.164, -0.419, -0.022, 0.702],
    [2429629.5, 2431456.5, 24.426, 1.645, 2.086, -1.106],
    [2431456.5, 2433282.5, 27.050, 2.499, -1.232, 0.614],
    [2433282.5, 2434378.5, 28.932, 1.127, 0.220, -0.277],
    [2434378.5, 2435473.5, 30.002, 0.737, -0.610, 0.631],
    [2435473.5, 2436569.5, 30.760, 1.409, 1.282, -0.799],
    [2436569.5, 2437665.5, 32.652, 1.577, -1.115, 0.507],
    [2437665.5, 2438761.5, 33.621, 0.868, 0.406, 0.199],
    [2438761.5, 2439856.5, 35.093, 2.275, 1.002, -0.414],
    [2439856.5, 2440952.5, 37.956, 3.035, -0.242, 0.202],
    [2440952.5, 2442048.5, 40.951, 3.157, 0.364, -0.229],
    [2442048.5, 2443144.5, 44.244, 3.198, -0.323, 0.172],
    [2443144.5, 2444239.5, 47.291, 3.069, 0.193, -0.192],
    [2444239.5, 2445335.5, 50.361, 2.878, -0.384, 0.081],
    [2445335.5, 2446431.5, 52.936, 2.354, -0.140, -0.166],
    [2446431.5, 2447527.5, 54.984, 1.577, -0.637, 0.448],
    [2447527.5, 2448622.5, 56.373, 1.649, 0.709, -0.277],
    [2448622.5, 2449718.5, 58.453, 2.235, -0.122, 0.111],
    [2449718.5, 2450814.5, 60.677, 2.324, 0.212, -0.315],
    [2450814.5, 2451910.5, 62.899, 1.804, -0.732, 0.112],
    [2451910.5, 2453005.5, 64.082, 0.675, -0.396, 0.193],
    [2453005.5, 2454101.5, 64.555, 0.463, 0.184, -0.008],
    [2454101.5, 2455197.5, 65.194, 0.809, 0.161, -0.101],
    [2455197.5, 2456293.5, 66.063, 0.828, -0.142, 0.168],
    [2456293.5, 2457388.5, 66.917, 1.046, 0.360, -0.282],
];

/// Stephenson et al. (2016) 长期项公式，用于公元前 720 年以前
fn deltat_longterm_morrison_stephenson(tjd: f64) -> f64 {
    let ygreg = 2000.0 + (tjd - J2000) / 365.2425;
    let u = (ygreg - 1820.0) / 100.0;
    -20.0 + 32.0 * u * u
}

/// 使用 Stephenson et al. (2016) 样条插值表计算 ΔT（秒）
fn deltat_from_dtcf16(tjd: f64, tid_acc: f64) -> Option<f64> {
    for (idx, row) in DTCF16.iter().enumerate() {
        let jd_start = row[0];
        let jd_end = row[1];

        if tjd >= jd_start && tjd < jd_end {
            let t = (tjd - jd_start) / (jd_end - jd_start);
            let dt = row[2] + row[3] * t + row[4] * t * t + row[5] * t * t * t;
            let ygreg = 2000.0 + (tjd - J2000) / 365.2425;

            // 样条表不同行使用不同的潮汐加速度参数
            // 第 0-14 行（公元前 720 年 - 公元 1865 年）：使用 -25.95
            // 第 15 行及以后（公元 1865-2016 年）：使用 -25.80（DE431，与 swetest 一致）
            let use_tid_acc = if idx <= 14 { SE_TIDAL_ANCIENT } else { SE_TIDAL_DE431 };
            return Some(adjust_for_tidacc(dt, ygreg, use_tid_acc, SE_TIDAL_26, true));
        }
    }
    None
}

/// Stephenson et al. (2016) 公式，用于公元前 720 年到 2016 年
fn deltat_stephenson_etc_2016(tjd: f64, tid_acc: f64) -> f64 {
    let ygreg = 2000.0 + (tjd - J2000) / 365.2425;
    
    // 首先尝试使用样条插值表（公元前 720 年到 2016 年）
    if let Some(dt) = deltat_from_dtcf16(tjd, tid_acc) {
        return dt;
    }
    
    // 对于公元前 720 年以前，使用长期抛物线公式
    if ygreg < -720.0 {
        let t = (ygreg - 1825.0) / 100.0;
        let mut dt = -320.0 + 32.5 * t * t;
        dt -= 179.7337208; // 连续化修正
        // 古代年份使用不同的潮汐加速度参数
        return adjust_for_tidacc(dt, ygreg, SE_TIDAL_ANCIENT, SE_TIDAL_26, true);
    }
    
    // 2016 年以后，使用外推（这个情况不应该在这里发生）
    let t = (ygreg - 1825.0) / 100.0;
    let mut dt = -320.0 + 32.5 * t * t;
    dt += 269.4790417; // 连续化修正
    adjust_for_tidacc(dt, ygreg, tid_acc, SE_TIDAL_26, true)
}

/// Espenak & Meeus (2006) 公式，直接从年份计算（秒）
fn deltat_espenak_meeus_1620(tjd: f64, tid_acc: f64) -> f64 {
    let ygreg = 2000.0 + (tjd - J2000) / 365.2425;
    let ans = if ygreg < -500.0 {
        deltat_longterm_morrison_stephenson(tjd)
    } else if ygreg < 500.0 {
        let u = ygreg / 100.0;
        ((((((0.0090316521 * u + 0.022174192) * u - 0.1798452) * u - 5.952053) * u + 33.78311) * u - 1014.41) * u + 10583.6)
    } else if ygreg < 1600.0 {
        let u = (ygreg - 1000.0) / 100.0;
        ((((((0.0083572073 * u - 0.005050998) * u - 0.8503463) * u + 0.319781) * u + 71.23472) * u - 556.01) * u + 1574.2)
    } else if ygreg < 1700.0 {
        let u = ygreg - 1600.0;
        120.0 - 0.9808 * u - 0.01532 * u * u + u * u * u / 7129.0
    } else if ygreg < 1800.0 {
        let u = ygreg - 1700.0;
        (((-u / 1174000.0 + 0.00013336) * u - 0.0059285) * u + 0.1603) * u + 8.83
    } else if ygreg < 1860.0 {
        let u = ygreg - 1800.0;
        ((((((0.000000000875 * u - 0.0000001699) * u + 0.0000121272) * u - 0.00037436) * u + 0.0041116) * u + 0.0068612) * u - 0.332447) * u + 13.72
    } else if ygreg < 1900.0 {
        let u = ygreg - 1860.0;
        ((((u / 233174.0 - 0.0004473624) * u + 0.01680668) * u - 0.251754) * u + 0.5737) * u + 7.62
    } else if ygreg < 1920.0 {
        let u = ygreg - 1900.0;
        (((-0.000197 * u + 0.0061966) * u - 0.0598939) * u + 1.494119) * u - 2.79
    } else if ygreg < 1941.0 {
        let u = ygreg - 1920.0;
        21.20 + 0.84493 * u - 0.076100 * u * u + 0.0020936 * u * u * u
    } else if ygreg < 1961.0 {
        let u = ygreg - 1950.0;
        29.07 + 0.407 * u - u * u / 233.0 + u * u * u / 2547.0
    } else if ygreg < 1986.0 {
        let u = ygreg - 1975.0;
        45.45 + 1.067 * u - u * u / 260.0 - u * u * u / 718.0
    } else if ygreg < 2005.0 {
        let u = ygreg - 2000.0;
        ((((0.00002373599 * u + 0.000651814) * u + 0.0017275) * u - 0.060374) * u + 0.3345) * u + 63.86
    } else {
        // 2005 年以后，使用外推
        let dy = (ygreg - 1820.0) / 100.0;
        -20.0 + 31.0 * dy * dy
    };
    
    adjust_for_tidacc(ans, ygreg, tid_acc, SE_TIDAL_26, false)
}

/// DT_AT 表（1620 年 - 2050 年）
/// 从 constants::DT_AT 读取

/// 使用线性插值计算 DT_AT 表的 ΔT（秒）
/// 注意：DT_AT 表不是等间距的，使用简单的线性插值
fn deltat_aa(tjd: f64, tid_acc: f64) -> f64 {
    let d = &constants::DT_AT;
    
    // 计算年份（使用 365.25 天/年）
    let y = 2000.0 + (tjd - 2451544.5) / 365.2425;
    
    // 提取年份和 DT 值到临时数组
    let mut years = Vec::new();
    let mut dt_vals = Vec::new();
    for i in (0..d.len()).step_by(2) {
        if i >= d.len() {
            break;
        }
        years.push(d[i]);
        dt_vals.push(d[i + 1]);
    }
    
    let tabsiz = years.len();
    let tabend = years[tabsiz - 1]; // 表尾年份（2028）

    // 查找 y 所在的区间
    if y < years[0] {
        // y 在表前（1620 年以前），使用外推公式
        let b = y - 2000.0;
        let ans = if y < 2500.0 {
            b * b * b * 121.0 / 30000000.0 + b * b / 1250.0 + b * 521.0 / 3000.0 + 64.0
        } else {
            let b2 = 0.01 * (y - 2000.0);
            b2 * b2 * 32.5 + 42.5
        };
        return ans;
    }
    
    if y > tabend {
        // y 在表后（2028 年以后），使用外推公式 + 慢过渡
        let b = y - 2000.0;
        let ans = if y < 2500.0 {
            b * b * b * 121.0 / 30000000.0 + b * b / 1250.0 + b * 521.0 / 3000.0 + 64.0
        } else {
            let b2 = 0.01 * (y - 2000.0);
            b2 * b2 * 32.5 + 42.5
        };
        
        // 慢过渡处理（2028-2128 年，100 年过渡期），与 swetest 一致
        if y <= tabend + 100.0 {
            let ans3 = dt_vals[tabsiz - 1]; // 表尾的 DT 值
            let b2 = tabend - 2000.0;
            let ans2 = b2 * b2 * b2 * 121.0 / 30000000.0 + b2 * b2 / 1250.0 + b2 * 521.0 / 3000.0 + 64.0;
            let dd = ans2 - ans3;
            return ans + dd * (y - (tabend + 100.0)) * 0.01;
        }
        
        return ans;
    }

    // y 在表范围内（1620-2028 年），进行线性插值
    let mut iy = 0;
    for i in 0..tabsiz - 1 {
        if (y.round() as i32) >= years[i] as i32 && (y.round() as i32) < years[i + 2] as i32 {
            iy = i;
            break;
        }
    }
    
    // 线性插值
    let t = (y - years[iy]) / (years[iy + 2] - years[iy]);
    let ans = dt_vals[iy] + t * (dt_vals[iy + 2] - dt_vals[iy]);

    // DT_AT 表已经是观测数据，不进行潮汐调整
    ans
}

/// 计算 ΔT（ET - UT），单位：天
/// 完整移植 swetest 的 calc_deltat 函数
pub fn calc_deltat(tjd: f64) -> f64 {
    // 默认潮汐加速度（DE431，与 swetest 一致）
    let tid_acc = SE_TIDAL_DE431;  // -25.80

    // Stephenson/Morrison/Hohenkerk 2016 模型（默认）
    // 用于 1955 年 1 月 1 日以前（JD 2435108.5）
    const JD_1955: f64 = 2435108.5;
    const JD_TRANSITION: f64 = 2434108.5;

    if tjd < JD_1955 {
        let mut deltat = deltat_stephenson_etc_2016(tjd, tid_acc) / 86400.0;

        // 过渡修正（1000 天线性过渡）
        if tjd >= JD_TRANSITION {
            deltat += (1.0 - (JD_1955 - tjd) / 1000.0) * 0.6610218 / 86400.0;
        }

        return deltat;
    }

    // 1955 年以后，使用 DT_AT 表（Astronomical Almanac / IERS 数据）
    deltat_aa(tjd, tid_acc) / 86400.0
}

/// TD-UT 计算
///
/// 注意是修正儒略日之间误差，内部使用
/// # Argument
/// - `t`: 相对于 J2000 的天数
pub fn dt_t(t: f64) -> f64 {
    let tjd = t + J2000;
    calc_deltat(tjd)
}

#[test]
fn test_dt_t() {
    println!("{:?}", dt_t(3062.49987811566)); // 0.0007605955088259062
}

/// 坐标系转换

/// 直角转为球坐标
///
/// # Argument
///
/// - `z`: 直角坐标的（x,y,z）
/// - `returns`: $(\theta, \phi, t)$ 注意，这个半径不是第一个返回值
///
pub fn xyz2llr(z:(f64,f64,f64)) -> (f64, f64, f64) {
    let (x, y,z) =z;
    let r = (x.powi(2) + y.powi(2) + z.powi(2)).sqrt();
    let theta = (z / r).asin();
    let phi = y.atan2(x);
    (rad2mrad(phi), theta, r)
}

/// 球面转直角
///
/// # Argument
/// - `jw`: $(\theta, \phi, r)$ 球面坐标系
pub fn llr2xyz(jw: (f64,f64,f64)) -> (f64,f64,f64) {
    let (j, w, r_val)= jw;
    let x = r_val * w.cos() * j.cos();
    let y = r_val * w.cos() * j.sin();
    let z = r_val * w.sin();
    (x,y,z)
}

/// 球面坐标旋转
pub fn llr_conv(jw:(f64,f64,f64), e: f64) -> (f64, f64,f64) {
    let (j,w, r2) =jw;
    let r0 = (j.sin() * e.cos() - w.tan() * e.sin()).atan2(j.cos());
    let r1 = (e.cos() * w.sin() + e.sin() * w.cos() * j.sin()).asin();
    let r0 = rad2mrad(r0);
    (r0, r1,r2)
}


/// 将角度转为 $ 0-2\pi $之间
pub fn rad2mrad(rad: f64) -> f64 {
    let r = rad - (rad / (2.0 * PI)).floor() * 2.0 * PI;
    let r = if r < 0.0 { r + 2.0 * PI } else { r };
    r
}


/// 将角度转为$-\pi-\pi$之间
///
pub fn rad2rrad( v: f64) -> f64 {
    // 对超过 -PI 到 PI 的角度转为 -PI 到 PI
    let v = v % (2.0 * PI);
    if v <= -PI {
        return v + 2.0 * PI;
    }
    if v > PI {
        return v - 2.0 * PI;
    }
    v
}
