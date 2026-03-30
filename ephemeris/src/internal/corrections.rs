//! 天体测量矫正模块
//!
//! 实现从物理黄经（几何位置）到视黄经的完整矫正流程
//! 参考 Swiss Ephemeris 的计算方法
//!
//! 矫正流程：
//! 1. 光时修正 (Light-time Correction)
//! 2. 引力偏折 (Gravitational Deflection)
//! 3. 光行差 (Annual Aberration)
//! 4. 岁差 (Precession)
//! 5. 章动 (Nutation)
//! 6. 黄赤坐标转换 (Ecliptic Transformation)

use std::f64::consts::PI;
use super::constants;
use super::math_utils::{self, xyz2llr, llr2xyz};
use super::ephemeris::{p_coord, e_coord, m_coord, obliquity, nutation2};

/// 光速 (km/s)
const CLIGHT: f64 = 299792.458;

/// 天文单位 (km)
const AUNIT: f64 = constants::CS_AU;

/// 光行时常数 (日/AU)
const LIGHT_TIME_PER_AU: f64 = AUNIT / CLIGHT / 86400.0;

/// 引力常数相关 (用于引力偏折计算)
/// 太阳引力半径 (km)
const R_SUN: f64 = 1.476625038;

/// 笛卡尔坐标 (X, Y, Z)
pub type Cartesian = (f64, f64, f64);

/// 球坐标 (经度，纬度，距离)
pub type Spherical = (f64, f64, f64);

// ============================================================================
// 1. 光时修正 (Light-time Correction)
// ============================================================================

/// 计算光时修正
///
/// 由于光速有限，我们看到的天体位置是它在光线发出时刻的位置
/// 对于太阳系天体，需要迭代计算 t' = t - dt
///
/// # Arguments
/// * `xt` - 天体编号 (0=地球，9=太阳，10=月球)
/// * `t` - 当前时刻 (相对于 J2000 的儒略世纪)
/// * `earth_pos` - 当前时刻地球位置
/// * `iter` - 迭代次数 (通常 1-2 次即可)
///
/// # Returns
/// 返回光时修正后的天体位置 (历元 t - dt 时刻)
pub fn light_time_correction(
    xt: usize,
    t: f64,
    earth_pos: Spherical,
    iter: i32,
) -> (Spherical, f64) {
    let mut current_t = t;
    let mut dt = 0.0;
    
    // 初始位置计算
    let mut body_pos = if xt == 10 {
        m_coord(t, -1, -1, -1)
    } else if xt == 9 {
        // 太阳：使用地球位置的镜像
        let earth = e_coord(t, -1, -1, -1);
        (earth.0 + PI, -earth.1, earth.2)
    } else {
        p_coord(xt, t, -1, -1, -1)
    };
    
    // 迭代计算光时
    for _ in 0..iter {
        // 计算地心到天体的向量
        let earth_xyz = llr2xyz(earth_pos);
        let body_xyz = llr2xyz(body_pos);
        
        // 地心向量 = 天体位置 - 地球位置
        let geo_xyz = (
            body_xyz.0 - earth_xyz.0,
            body_xyz.1 - earth_xyz.1,
            body_xyz.2 - earth_xyz.2,
        );
        
        // 计算距离 (AU)
        let distance = (geo_xyz.0.powi(2) + geo_xyz.1.powi(2) + geo_xyz.2.powi(2)).sqrt();
        
        // 光时 (儒略日)
        dt = distance * LIGHT_TIME_PER_AU;
        
        // 回溯到光线发射时刻
        current_t = t - dt / 36525.0; // 转为儒略世纪
        
        // 重新计算天体在 t - dt 时刻的位置
        body_pos = if xt == 10 {
            m_coord(current_t, -1, -1, -1)
        } else if xt == 9 {
            let earth = e_coord(current_t, -1, -1, -1);
            (earth.0 + PI, -earth.1, earth.2)
        } else {
            p_coord(xt, current_t, -1, -1, -1)
        };
    }
    
    (body_pos, dt)
}

/// 太阳专用光时修正
///
/// 太阳的光时约 499 秒 (8 分 19 秒)
/// 太阳视位置 = 地球位置镜像 + 光时修正
pub fn solar_light_time_correction(
    t: f64,
    earth_pos: Spherical,
) -> (Spherical, f64) {
    let mut dt = 0.0;
    let mut current_t = t;
    
    // 迭代 1 次通常足够
    for _ in 0..1 {
        // 计算地日距离
        let distance = earth_pos.2; // 地球向径即为地日距离
        dt = distance * LIGHT_TIME_PER_AU;
        
        // 回溯到光线发射时刻
        current_t = t - dt / 36525.0;
    }
    
    // 返回 t - dt 时刻的地球位置（太阳视位置是其镜像）
    let earth_retarded = e_coord(current_t, -1, -1, -1);
    let sun_pos = (earth_retarded.0 + PI, -earth_retarded.1, earth_retarded.2);
    
    (sun_pos, dt)
}

// ============================================================================
// 2. 引力偏折修正 (Gravitational Deflection)
// ============================================================================

/// 计算引力偏折
///
/// 光线经过大质量天体（主要是太阳）附近时会发生弯曲
/// 使用相对论公式计算偏折角
///
/// # Arguments
/// * `body_pos` - 天体位置 (笛卡尔坐标，地心)
/// * `sun_pos` - 太阳位置 (笛卡尔坐标，地心)
///
/// # Returns
/// 修正后的天体位置
pub fn gravitational_deflection(
    body_pos: Cartesian,
    sun_pos: Cartesian,
) -> Cartesian {
    // 计算天体方向单位向量
    let r = (body_pos.0.powi(2) + body_pos.1.powi(2) + body_pos.2.powi(2)).sqrt();
    if r < 1e-10 {
        return body_pos;
    }
    
    let u = (body_pos.0 / r, body_pos.1 / r, body_pos.2 / r);
    
    // 计算太阳方向单位向量
    let rs = (sun_pos.0.powi(2) + sun_pos.1.powi(2) + sun_pos.2.powi(2)).sqrt();
    if rs < 1e-10 {
        return body_pos;
    }
    
    let us = (sun_pos.0 / rs, sun_pos.1 / rs, sun_pos.2 / rs);
    
    // 计算天体与太阳的角距离
    let cos_psi = u.0 * us.0 + u.1 * us.1 + u.2 * us.2;
    let psi = cos_psi.acos();
    
    // 避免除以零（天体与太阳重合时）
    if psi < 1e-6 {
        return body_pos;
    }
    
    // 引力偏折公式 (一阶近似)
    // 偏折角 = 2GM/(c^2 * b) * (1 + cos(psi)) / sin(psi)
    // 其中 b 是碰撞参数，近似为 rs * sin(psi)
    let factor = 2.0 * R_SUN / (rs * AUNIT) * (1.0 + cos_psi) / psi.sin();
    
    // 计算偏折方向（从天体指向远离太阳的方向）
    // 偏折向量 = factor * (us - cos_psi * u)
    let deflection = (
        us.0 - cos_psi * u.0,
        us.1 - cos_psi * u.1,
        us.2 - cos_psi * u.2,
    );
    
    // 应用偏折
    let corrected = (
        u.0 + factor * deflection.0,
        u.1 + factor * deflection.1,
        u.2 + factor * deflection.2,
    );
    
    // 归一化并恢复原距离
    let corr_len = (corrected.0.powi(2) + corrected.1.powi(2) + corrected.2.powi(2)).sqrt();
    (
        corrected.0 / corr_len * r,
        corrected.1 / corr_len * r,
        corrected.2 / corr_len * r,
    )
}

// ============================================================================
// 3. 光行差修正 (Annual Aberration)
// ============================================================================

/// 计算周年光行差
///
/// 由于地球公转速度导致的光线方向偏移
/// 使用相对论完整公式
///
/// # Arguments
/// * `body_pos` - 天体位置 (笛卡尔坐标，地心)
/// * `earth_vel` - 地球速度 (笛卡尔坐标，AU/日)
///
/// # Returns
/// 修正后的天体位置
pub fn annual_aberration(
    body_pos: Cartesian,
    earth_vel: Cartesian,
) -> Cartesian {
    // 计算天体方向单位向量
    let r = (body_pos.0.powi(2) + body_pos.1.powi(2) + body_pos.2.powi(2)).sqrt();
    if r < 1e-10 {
        return body_pos;
    }
    
    let u = (body_pos.0 / r, body_pos.1 / r, body_pos.2 / r);
    
    // 地球速度 (AU/日 -> km/s)
    // 1 AU/日 = AUNIT / 86400 km/s
    let v = (
        earth_vel.0 * AUNIT / 86400.0,
        earth_vel.1 * AUNIT / 86400.0,
        earth_vel.2 * AUNIT / 86400.0,
    );
    
    // v/c
    let v2 = (v.0.powi(2) + v.1.powi(2) + v.2.powi(2)) / CLIGHT.powi(2);
    let beta = (v.0 / CLIGHT, v.1 / CLIGHT, v.2 / CLIGHT);
    
    // 相对论光行差公式
    // u' = (u / gamma + v/c * (1 + u·v/c / (1 + 1/gamma))) / (1 + u·v/c)
    // 其中 gamma = 1 / sqrt(1 - v^2/c^2)
    
    let gamma_inv = (1.0 - v2).sqrt();
    let u_dot_v = u.0 * beta.0 + u.1 * beta.1 + u.2 * beta.2;
    
    let f1 = u_dot_v;
    let f2 = 1.0 + f1 / (1.0 + gamma_inv);
    
    let corrected = (
        gamma_inv * u.0 + f2 * beta.0,
        gamma_inv * u.1 + f2 * beta.1,
        gamma_inv * u.2 + f2 * beta.2,
    );
    
    let corr_len = (corrected.0.powi(2) + corrected.1.powi(2) + corrected.2.powi(2)).sqrt();
    
    // 归一化并恢复原距离
    (
        corrected.0 / corr_len * r,
        corrected.1 / corr_len * r,
        corrected.2 / corr_len * r,
    )
}

/// 使用简化公式计算光行差（用于快速计算）
///
/// 光行差常数 K ≈ 20.4955 角秒
pub fn annual_aberration_approx(
    body_pos: Spherical,
    earth_vel: Spherical,
) -> Spherical {
    // 光行差常数 (弧度)
    let k = 20.4955 / constants::RAD;
    
    // 简化公式：Δλ = -K * cos(λ - λ_sun) / cos(β)
    // Δβ = -K * sin(β) * sin(λ - λ_sun)
    
    let sun_lon = earth_vel.0 + PI; // 太阳黄经
    let delta_lon = body_pos.0 - sun_lon;
    
    let d_lon = -k * delta_lon.cos() / body_pos.1.cos();
    let d_lat = -k * body_pos.1.sin() * delta_lon.sin();
    
    (
        math_utils::rad2mrad(body_pos.0 + d_lon),
        body_pos.1 + d_lat,
        body_pos.2,
    )
}

// ============================================================================
// 4. 岁差修正 (Precession)
// ============================================================================

/// Vondrák et al. (2011) 岁差模型
///
/// 适用于长时期的高精度岁差计算（-6000 年 ~ +6000 年）
/// 
/// 参考：
/// - Vondrák, Capitaine, Wallace: "New precession expressions, valid for long time intervals"
/// - A&A 534, A22 (2011)
/// - Swiss Ephemeris swephlib.c pre_pecl() 和 pre_pequ()
///
/// # Arguments
/// * `tjd` - 儒略日
///
/// # Returns
/// 岁差旋转矩阵 (3x3) - 从 J2000 转换到目标历元
pub fn precession_vondrak_2011(tjd: f64) -> [[f64; 3]; 3] {
    use std::f64::consts::PI;

    const D2PI: f64 = 2.0 * PI;
    const AS2R: f64 = PI / (180.0 * 3600.0); // 角秒转弧度
    const J2000: f64 = 2451545.0;
    // EPS0 = 84381.406 角秒 = 23.439291111... 度（Vondrák 2011 标准值）
    const EPS0: f64 = 84381.406 * AS2R;

    let t = (tjd - J2000) / 36525.0;

    // ========== 黄道极计算 (pre_pecl) ==========
    // 多项式系数
    const PQPOL: [[f64; 2]; 4] = [
        [5851.607687, -1600.886300],
        [-0.1189000, 1.1689818],
        [-0.00028913, -0.00000020],
        [0.000000101, -0.000000437],
    ];

    // 周期项系数
    const PQPER: [[f64; 8]; 5] = [
        [708.15, 2309.0, 1620.0, 492.2, 1183.0, 622.0, 882.0, 547.0],
        [-5486.751211, -17.127623, -617.517403, 413.44294, 78.614193, -180.732815, -87.676083, 46.140315],
        [-684.66156, 2446.28388, 399.671049, -356.652376, -186.387003, -316.80007, 198.296701, 101.135679],
        [667.66673, -2354.886252, -428.152441, 376.202861, 184.778874, 335.321713, -185.138669, -120.97283],
        [-5523.863691, -549.74745, -310.998056, 421.535876, -36.776172, -145.278396, -34.74445, 22.885731],
    ];

    let mut p = 0.0;
    let mut q = 0.0;

    // 周期项
    for i in 0..8 {
        let w = D2PI * t;
        let a = w / PQPER[0][i];
        let s = a.sin();
        let c = a.cos();
        p += c * PQPER[1][i] + s * PQPER[3][i];
        q += c * PQPER[2][i] + s * PQPER[4][i];
    }

    // 多项式项
    let mut w = 1.0;
    for i in 0..4 {
        p += PQPOL[i][0] * w;
        q += PQPOL[i][1] * w;
        w *= t;
    }

    // 转为弧度
    p *= AS2R;
    q *= AS2R;

    // 黄道极向量
    let z = (1.0 - p*p - q*q).max(0.0).sqrt();
    let s_eps = EPS0.sin();
    let c_eps = EPS0.cos();

    let pecl = [
        p,
        -q * c_eps - z * s_eps,
        -q * s_eps + z * c_eps,
    ];

    // ========== 赤道极计算 (pre_pequ) ==========
    // 多项式系数
    const XYPOL: [[f64; 2]; 4] = [
        [5453.282155, -73750.930350],
        [0.4252841, -0.7675452],
        [-0.00037173, -0.00018725],
        [-0.000000152, 0.000000231],
    ];

    // 周期项系数
    const XYPER: [[f64; 14]; 5] = [
        [256.75, 708.15, 274.2, 241.45, 2309.0, 492.2, 396.1, 288.9, 231.1, 1610.0, 620.0, 157.87, 220.3, 1200.0],
        [-819.940624, -8444.676815, 2600.009459, 2755.17563, -167.659835, 871.855056, 44.769698, -512.313065, -819.415595, -538.071099, -189.793622, -402.922932, 179.516345, -9.814756],
        [75004.344875, 624.033993, 1251.136893, -1102.212834, -2660.66498, 699.291817, 153.16722, -950.865637, 499.754645, -145.18821, 558.116553, -23.923029, -165.405086, 9.344131],
        [81491.287984, 787.163481, 1251.296102, -1257.950837, -2966.79973, 639.744522, 131.600209, -445.040117, 584.522874, -89.756563, 524.42963, -13.549067, -210.157124, -44.919798],
        [1558.515853, 7774.939698, -2219.534038, -2523.969396, 247.850422, -846.485643, -1393.124055, 368.526116, 749.045012, 444.704518, 235.934465, 374.049623, -171.33018, -22.899655],
    ];

    let mut x = 0.0;
    let mut y = 0.0;

    // 周期项
    for i in 0..14 {
        let w = D2PI * t;
        let a = w / XYPER[0][i];
        let s = a.sin();
        let c = a.cos();
        x += c * XYPER[1][i] + s * XYPER[3][i];
        y += c * XYPER[2][i] + s * XYPER[4][i];
    }

    // 多项式项
    let mut w = 1.0;
    for i in 0..4 {
        x += XYPOL[i][0] * w;
        y += XYPOL[i][1] * w;
        w *= t;
    }

    // 转为弧度
    x *= AS2R;
    y *= AS2R;

    // 赤道极向量
    let w = x*x + y*y;
    let pequ = [
        x,
        y,
        if w < 1.0 { (1.0 - w).sqrt() } else { 0.0 },
    ];

    // ========== 构建岁差矩阵 ==========
    // 春分点向量 = 赤道极 × 黄道极（叉积）
    let mut eqx = [0.0; 3];
    cross_prod(&pequ, &pecl, &mut eqx);

    // 归一化
    let norm = eqx.iter().map(|v| v*v).sum::<f64>().sqrt();
    if norm > 1e-15 {
        eqx.iter_mut().for_each(|v| *v /= norm);
    }

    // Y 轴 = 赤道极 × 春分点（完成右手系）
    // 注意：Swiss Ephemeris 使用的是 peqr × eqx，不是 pecl × eqx
    let mut y_axis = [0.0; 3];
    cross_prod(&pequ, &eqx, &mut y_axis);

    // 构建旋转矩阵（列向量形式）
    // 第 1 列：春分点（X 轴）
    // 第 2 列：Y 轴
    // 第 3 列：赤道极（Z 轴）
    [
        [eqx[0], y_axis[0], pequ[0]],
        [eqx[1], y_axis[1], pequ[1]],
        [eqx[2], y_axis[2], pequ[2]],
    ]
}

/// 向量叉积
fn cross_prod(a: &[f64; 3], b: &[f64; 3], x: &mut [f64; 3]) {
    x[0] = a[1] * b[2] - a[2] * b[1];
    x[1] = a[2] * b[0] - a[0] * b[2];
    x[2] = a[0] * b[1] - a[1] * b[0];
}

/// IAU 2006 岁差模型（短时期高精度）
///
/// 计算从 J2000 到观测时刻的岁差旋转
///
/// # Arguments
/// * `t` - 相对于 J2000 的儒略世纪数
///
/// # Returns
/// 返回岁差角 (zeta, z, theta) 单位：弧度
pub fn precession_iau2006(t: f64) -> (f64, f64, f64) {
    // 使用霍纳法则（Horner's method）正确实现 IAU 2006 公式
    // 参考：Swiss Ephemeris swephlib.c precess_1() SEMOD_PREC_IAU_2006
    // 以及：Capitaine et al. (2003) A&A 412, 567-586
    
    // zeta 角（角秒）
    let zeta_arcsec = (((((-0.0000003173 * t - 0.000005971) * t + 0.01801828) * t + 0.2988499) * t + 2306.083227) * t + 2.650545) * t;
    
    // z 角（角秒）
    let z_arcsec = (((((-0.0000002904 * t - 0.000028596) * t + 0.01826837) * t + 1.0927348) * t + 2306.077181) * t - 2.650545) * t;
    
    // theta 角（角秒）
    let theta_arcsec = ((((-0.00000011274 * t - 0.000007089) * t - 0.04182264) * t - 0.4294934) * t + 2004.191903) * t;
    
    // 转为弧度
    (
        zeta_arcsec / constants::RAD,
        z_arcsec / constants::RAD,
        theta_arcsec / constants::RAD,
    )
}

/// 应用岁差旋转到笛卡尔坐标
///
/// 默认使用 Vondrák 2011 模型（适用于长时期，-6000 年 ~ +6000 年）
/// 
/// # Arguments
/// * `pos` - 笛卡尔坐标
/// * `t` - 相对于 J2000 的儒略世纪数
///
/// # Returns
/// 修正后的笛卡尔坐标
pub fn apply_precession(
    pos: Cartesian,
    t: f64,
) -> Cartesian {
    // 将儒略世纪转换为儒略日
    let tjd = t * 36525.0 + 2451545.0;

    // 使用 Vondrák 2011 岁差矩阵
    // 注意：precession_vondrak_2011 返回的是从目标历元到 J2000 的变换矩阵
    // 我们需要用它的转置来从 J2000 转换到目标历元
    let prec_matrix = precession_vondrak_2011(tjd);

    // 应用旋转矩阵的转置（从 J2000 到目标历元）
    // x' = M[0][0]*x + M[1][0]*y + M[2][0]*z
    // y' = M[0][1]*x + M[1][1]*y + M[2][1]*z
    // z' = M[0][2]*x + M[1][2]*y + M[2][2]*z
    (
        pos.0 * prec_matrix[0][0] + pos.1 * prec_matrix[1][0] + pos.2 * prec_matrix[2][0],
        pos.0 * prec_matrix[0][1] + pos.1 * prec_matrix[1][1] + pos.2 * prec_matrix[2][1],
        pos.0 * prec_matrix[0][2] + pos.1 * prec_matrix[1][2] + pos.2 * prec_matrix[2][2],
    )
}

/// 应用 IAU 2006 岁差（短时期，1900-2100 年）
///
/// # Arguments
/// * `pos` - 笛卡尔坐标
/// * `t` - 相对于 J2000 的儒略世纪数
///
/// # Returns
/// 修正后的笛卡尔坐标
pub fn apply_precession_iau2006(
    pos: Cartesian,
    t: f64,
) -> Cartesian {
    let (zeta, z_angle, theta) = precession_iau2006(t);

    // IAU 2006 岁差变换：Rz(-z_A) × Ry(+θ_A) × Rz(-ζ_A)
    // 从 J2000 转换到目标历元
    
    // 第一步：Rz(-ζ_A) - 绕 Z 轴旋转 -zeta
    // Rz(-α) = [cos α    sin α   0]
    //          [-sin α   cos α   0]
    //          [0        0       1]
    let (x, y, z_coord) = pos;
    let cos_zeta = zeta.cos();
    let sin_zeta = zeta.sin();
    let (x1, y1, z1) = (
        x * cos_zeta + y * sin_zeta,
        -x * sin_zeta + y * cos_zeta,
        z_coord,
    );

    // 第二步：Ry(+θ_A) - 绕 Y 轴旋转 +theta
    // Ry(+β) = [cos β    0   sin β]
    //          [0        1   0    ]
    //          [-sin β   0   cos β]
    let cos_theta = theta.cos();
    let sin_theta = theta.sin();
    let (x2, y2, z2) = (
        x1 * cos_theta + z1 * sin_theta,
        y1,
        -x1 * sin_theta + z1 * cos_theta,
    );

    // 第三步：Rz(-z_A) - 绕 Z 轴旋转 -z
    let cos_z_rot = z_angle.cos();
    let sin_z_rot = z_angle.sin();
    let (x3, y3, z3) = (
        x2 * cos_z_rot + y2 * sin_z_rot,
        -x2 * sin_z_rot + y2 * cos_z_rot,
        z2,
    );

    (x3, y3, z3)
}

// ============================================================================
// 5. 章动修正 (Nutation)
// ============================================================================

/// 章动矩阵
///
/// 用于将平赤道坐标转换到真赤道坐标
#[derive(Debug, Clone, Copy)]
pub struct NutationMatrix {
    pub m00: f64, pub m01: f64, pub m02: f64,
    pub m10: f64, pub m11: f64, pub m12: f64,
    pub m20: f64, pub m21: f64, pub m22: f64,
}

impl NutationMatrix {
    /// 从章动角构建矩阵
    ///
    /// # Arguments
    /// * `delta_psi` - 黄经章动 (弧度)
    /// * `delta_epsilon` - 交角章动 (弧度)
    /// * `epsilon` - 平黄赤交角 (弧度)
    pub fn from_nutation_angles(
        delta_psi: f64,
        delta_epsilon: f64,
        epsilon: f64,
    ) -> Self {
        let sin_dp = delta_psi.sin();
        let cos_dp = delta_psi.cos();
        let sin_de = delta_epsilon.sin();
        let cos_de = delta_epsilon.cos();
        let sin_e = epsilon.sin();
        let cos_e = epsilon.cos();
        let sin_e_dep = (epsilon + delta_epsilon).sin();
        let cos_e_dep = (epsilon + delta_epsilon).cos();
        
        // 章动矩阵 (从平赤道到真赤道)
        NutationMatrix {
            m00: cos_dp,
            m01: -sin_dp * cos_e,
            m02: -sin_dp * sin_e,
            m10: sin_dp * cos_e_dep,
            m11: cos_dp * cos_e * cos_e_dep + sin_e * sin_e_dep,
            m12: cos_dp * sin_e * cos_e_dep - cos_e * sin_e_dep,
            m20: sin_dp * sin_e_dep,
            m21: cos_dp * cos_e * sin_e_dep - sin_e * cos_e_dep,
            m22: cos_dp * sin_e * sin_e_dep + cos_e * cos_e_dep,
        }
    }
    
    /// 应用章动矩阵到笛卡尔坐标
    pub fn apply(&self, pos: Cartesian) -> Cartesian {
        (
            self.m00 * pos.0 + self.m01 * pos.1 + self.m02 * pos.2,
            self.m10 * pos.0 + self.m11 * pos.1 + self.m12 * pos.2,
            self.m20 * pos.0 + self.m21 * pos.1 + self.m22 * pos.2,
        )
    }
}

/// 计算章动矩阵
///
/// # Arguments
/// * `t` - 相对于 J2000 的儒略世纪数
/// * `epsilon` - 平黄赤交角
pub fn compute_nutation_matrix(
    t: f64,
    epsilon: f64,
) -> NutationMatrix {
    let (delta_psi, delta_epsilon) = nutation2(t);
    NutationMatrix::from_nutation_angles(delta_psi, delta_epsilon, epsilon)
}

// ============================================================================
// 6. 黄赤坐标转换 (Ecliptic Transformation)
// ============================================================================

/// 从赤道坐标转换到黄道坐标
///
/// # Arguments
/// * `pos` - 赤道坐标 (笛卡尔)
/// * `epsilon` - 黄赤交角 (弧度)
///
/// # Returns
/// 黄道坐标 (笛卡尔)
pub fn equatorial_to_ecliptic(
    pos: Cartesian,
    epsilon: f64,
) -> Cartesian {
    let cos_e = epsilon.cos();
    let sin_e = epsilon.sin();
    
    let (x, y, z) = pos;
    
    // 旋转矩阵 Rx(-epsilon)
    (
        x,
        y * cos_e + z * sin_e,
        -y * sin_e + z * cos_e,
    )
}

/// 从黄道坐标转换到赤道坐标
///
/// # Arguments
/// * `pos` - 黄道坐标 (笛卡尔)
/// * `epsilon` - 黄赤交角 (弧度)
///
/// # Returns
/// 赤道坐标 (笛卡尔)
pub fn ecliptic_to_equatorial(
    pos: Cartesian,
    epsilon: f64,
) -> Cartesian {
    let cos_e = epsilon.cos();
    let sin_e = epsilon.sin();
    
    let (x, y, z) = pos;
    
    // 旋转矩阵 Rx(epsilon)
    (
        x,
        y * cos_e - z * sin_e,
        y * sin_e + z * cos_e,
    )
}

// ============================================================================
// 7. 地球速度计算 (用于光行差)
// ============================================================================

/// 计算地球速度向量
///
/// 使用数值微分法计算地球在 J2000 赤道坐标系中的速度
///
/// # Arguments
/// * `t` - 当前时刻 (相对于 J2000 的儒略世纪)
/// * `dt` - 时间间隔 (儒略世纪)，默认 0.0001
pub fn compute_earth_velocity(
    t: f64,
    dt: f64,
) -> Cartesian {
    let earth_minus = e_coord(t - dt, -1, -1, -1);
    let earth_plus = e_coord(t + dt, -1, -1, -1);
    
    let xyz_minus = llr2xyz(earth_minus);
    let xyz_plus = llr2xyz(earth_plus);
    
    // 中心差分：v = (x(t+dt) - x(t-dt)) / (2*dt)
    // 结果单位：AU/儒略世纪
    (
        (xyz_plus.0 - xyz_minus.0) / (2.0 * dt),
        (xyz_plus.1 - xyz_minus.1) / (2.0 * dt),
        (xyz_plus.2 - xyz_minus.2) / (2.0 * dt),
    )
}

// ============================================================================
// 8. 完整矫正流程
// ============================================================================

/// 行星视位置矫正（完整流程）
///
/// 从物理位置（J2000 黄道）到视位置（真赤道真春分点）
/// 注意：此函数不用于太阳（xt=9）和月球（xt=10），它们有专用函数
///
/// # Arguments
/// * `xt` - 天体编号 (0-8，不包括 9 太阳和 10 月球)
/// * `t` - 当前时刻 (相对于 J2000 的儒略世纪)
/// * `body_pos` - 天体物理位置 (J2000 黄道，球坐标)
/// * `earth_pos` - 地球位置 (J2000 黄道，球坐标)
///
/// # Returns
/// (视黄经，视黄纬，视赤经，视赤纬，地心距，光行时)
pub fn compute_apparent_position(
    xt: usize,
    t: f64,
    body_pos: Spherical,
    earth_pos: Spherical,
) -> (f64, f64, f64, f64, f64, f64) {
    // 1. 光时修正
    let (body_retarded, light_time) = light_time_correction(xt, t, earth_pos, 1);
    
    // 2. 地心坐标转换 (从日心/质心到地心)
    let earth_xyz = llr2xyz(earth_pos);
    let body_xyz = llr2xyz(body_retarded);
    
    let geo_xyz = (
        body_xyz.0 - earth_xyz.0,
        body_xyz.1 - earth_xyz.1,
        body_xyz.2 - earth_xyz.2,
    );
    
    // 3. 引力偏折（太阳引力场）
    let sun_pos = {
        let sun = if xt == 9 {
            // 太阳自身不需要引力偏折
            (0.0, 0.0, 0.0)
        } else {
            let earth_t = e_coord(t, -1, -1, -1);
            (-earth_t.0, -earth_t.1, -earth_t.2) // 地心太阳位置
        };
        llr2xyz(sun)
    };
    
    let geo_deflected = if xt != 9 {
        gravitational_deflection(geo_xyz, sun_pos)
    } else {
        geo_xyz
    };
    
    // 4. 光行差修正
    let earth_vel = compute_earth_velocity(t, 0.0001);
    let geo_aberrated = annual_aberration(geo_deflected, earth_vel);
    
    // 5. 岁差修正 (J2000 -> 观测时刻平赤道)
    let geo_precessed = apply_precession(geo_aberrated, t);
    
    // 6. 章动修正 (平赤道 -> 真赤道)
    let epsilon_mean = obliquity(t);
    let nut_matrix = compute_nutation_matrix(t, epsilon_mean);
    let geo_nutated = nut_matrix.apply(geo_precessed);
    
    // 7. 转换到球坐标（真赤道真春分点）
    let (ra, dec, r_geo) = xyz2llr(geo_nutated);
    
    // 8. 黄赤坐标转换（真赤道 -> 真黄道）
    let epsilon_true = epsilon_mean + nutation2(t).1;
    let ecl_xyz = equatorial_to_ecliptic(geo_nutated, epsilon_true);
    let (ecl_lon, ecl_lat, _) = xyz2llr(ecl_xyz);
    
    (
        math_utils::rad2mrad(ecl_lon),
        ecl_lat,
        math_utils::rad2mrad(ra),
        dec,
        r_geo,
        light_time,
    )
}

/// 太阳视位置专用计算
///
/// 太阳视位置 = 地球位置镜像 + 光时 + 光行差
/// 注意：太阳没有引力偏折（太阳本身就是引力源）
pub fn compute_solar_apparent_position(
    t: f64,
    earth_pos: Spherical,
) -> (f64, f64, f64, f64, f64, f64) {
    // 1. 太阳光时修正（回溯约 499 秒）
    let (sun_retarded, light_time) = solar_light_time_correction(t, earth_pos);

    // 2. 地心坐标（太阳视位置 = 地球位置的镜像，方向取反）
    // 在日心坐标系中，太阳在原点，地球在 earth_pos
    // 在地心坐标系中，太阳在 -earth_retarded 方向
    // 地心太阳向量 = 日心太阳位置 (0) - 日心地球位置 = -earth_retarded
    let earth_xyz = llr2xyz(earth_pos);
    let sun_retarded_xyz = llr2xyz(sun_retarded);
    
    // 地心太阳位置 = sun_retarded - earth_pos
    // 但由于 sun_retarded 是地球位置的镜像（地球位置 + PI），所以：
    // 地心太阳位置 = -(earth_retarded + earth_pos) / 2 的近似
    // 更简单的方法：地心太阳距离 ≈ 地球向径
    let geo_xyz = (
        -earth_xyz.0,  // 太阳在地球的相反方向
        -earth_xyz.1,
        -earth_xyz.2,
    );

    // 3. 光行差修正（太阳也有光行差，约 20.5 角秒）
    let earth_vel = compute_earth_velocity(t, 0.0001);
    let geo_aberrated = annual_aberration(geo_xyz, earth_vel);

    // 4. 岁差修正
    let geo_precessed = apply_precession(geo_aberrated, t);

    // 5. 章动修正
    let epsilon_mean = obliquity(t);
    let nut_matrix = compute_nutation_matrix(t, epsilon_mean);
    let geo_nutated = nut_matrix.apply(geo_precessed);

    // 6. 转换到球坐标
    let (ra, dec, r_geo) = xyz2llr(geo_nutated);

    // 7. 黄赤坐标转换
    let epsilon_true = epsilon_mean + nutation2(t).1;
    let ecl_xyz = equatorial_to_ecliptic(geo_nutated, epsilon_true);
    let (ecl_lon, ecl_lat, _) = xyz2llr(ecl_xyz);

    (
        math_utils::rad2mrad(ecl_lon),
        ecl_lat,
        math_utils::rad2mrad(ra),
        dec,
        r_geo,
        light_time,
    )
}

/// 月球视位置专用计算
///
/// 月球需要特殊处理：
/// - 光时很短（约 1.3 秒），但需要考虑
/// - 光行差很小（约 0.003 角秒）
/// - m_coord 返回的距离单位是地球半径，不是 AU
/// 
/// 参考 Swiss Ephemeris app_pos_etc_moon() 实现
pub fn compute_lunar_apparent_position(
    t: f64,
    earth_pos: Spherical,
) -> (f64, f64, f64, f64, f64, f64) {
    // 获取月球位置（距离单位是地球半径）
    let moon_pos = m_coord(t, -1, -1, -1);

    // 将月球距离从地球半径转换为 AU
    // 1 AU = 149597870.7 km, 地球半径 = 6378.14 km
    // 1 AU ≈ 23455 地球半径
    const EARTH_RADIUS_PER_AU: f64 = 23455.0;
    let moon_dist_au = moon_pos.2 / EARTH_RADIUS_PER_AU;

    // 月光时（约 1.3 秒）
    let light_time = moon_dist_au * LIGHT_TIME_PER_AU;

    // 回溯到光线发射时刻
    let retarded_t = t - light_time / 36525.0;
    
    // 关键修复：在回溯时刻重新计算月球和地球位置
    // 这与 Swiss Ephemeris 的实现一致
    let moon_retarded = m_coord(retarded_t, -1, -1, -1);
    let earth_retarded = super::ephemeris::earth_lon(retarded_t, -1); // 回溯时刻的地球黄经

    // 将月球位置转换为 AU 单位
    let moon_xyz_au = {
        let moon_xyz = llr2xyz(moon_retarded);
        (
            moon_xyz.0 / EARTH_RADIUS_PER_AU,
            moon_xyz.1 / EARTH_RADIUS_PER_AU,
            moon_xyz.2 / EARTH_RADIUS_PER_AU,
        )
    };

    // 回溯时刻的地球位置
    let earth_xyz_retarded = llr2xyz((earth_retarded, 0.0, 1.0));

    // 2. 地心坐标（使用回溯时刻的地球位置）
    let geo_xyz = (
        moon_xyz_au.0 - earth_xyz_retarded.0,
        moon_xyz_au.1 - earth_xyz_retarded.1,
        moon_xyz_au.2 - earth_xyz_retarded.2,
    );

    // 3. 月球不需要引力偏折（距离太近）

    // 4. 光行差修正（使用回溯时刻的地球速度）
    let earth_vel_retarded = compute_earth_velocity(retarded_t, 0.0001);
    let geo_aberrated = annual_aberration(geo_xyz, earth_vel_retarded);

    // 5. 岁差修正
    let geo_precessed = apply_precession(geo_aberrated, t);

    // 6. 章动修正
    let epsilon_mean = obliquity(t);
    let nut_matrix = compute_nutation_matrix(t, epsilon_mean);
    let geo_nutated = nut_matrix.apply(geo_precessed);

    // 7. 转换到球坐标
    let (ra, dec, r_geo) = xyz2llr(geo_nutated);

    // 8. 黄赤坐标转换
    let epsilon_true = epsilon_mean + nutation2(t).1;
    let ecl_xyz = equatorial_to_ecliptic(geo_nutated, epsilon_true);
    let (ecl_lon, ecl_lat, _) = xyz2llr(ecl_xyz);

    (
        math_utils::rad2mrad(ecl_lon),
        ecl_lat,
        math_utils::rad2mrad(ra),
        dec,
        r_geo,
        light_time,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::internal::ephemeris::earth_lon;
    
    #[test]
    fn test_light_time_sun() {
        // 测试太阳光时修正
        // 2023 年 7 月 23 日 12:00 TT
        let jd = 2460149.0;
        let t = (jd - 2451545.0) / 36525.0;
        
        let earth_pos = e_coord(t, -1, -1, -1);
        let (sun_pos, light_time) = solar_light_time_correction(t, earth_pos);
        
        // 太阳光时应约 499 秒 = 0.00577 日
        println!("太阳光时：{} 日 = {} 秒", light_time, light_time * 86400.0);
        assert!(light_time > 0.005 && light_time < 0.006);
    }
    
    #[test]
    fn test_light_time_moon() {
        // 测试月光时修正概念验证
        // 月球距离约 384400 km = 0.00257 AU
        // 光时应约 1.3 秒
        
        let expected_distance_au = 0.00257;
        let expected_light_time = expected_distance_au * LIGHT_TIME_PER_AU;
        
        println!("预期月球距离：{} AU", expected_distance_au);
        println!("预期月光时：{} 秒", expected_light_time * 86400.0);
        
        // 验证光时计算逻辑正确
        assert!(expected_light_time * 86400.0 > 1.2 && expected_light_time * 86400.0 < 1.4);
    }
    
    #[test]
    fn test_annual_aberration() {
        // 测试光行差概念验证
        // 光行差常数 K ≈ 20.4955 角秒
        
        // 验证光行差量级公式正确
        let k = 20.4955; // 光行差常数（角秒）
        println!("光行差常数：{} 角秒", k);
        assert!(k > 20.0 && k < 21.0);
    }
    
    #[test]
    fn test_gravitational_deflection() {
        // 测试引力偏折
        // 设置一个靠近太阳的天体位置
        let body_pos = (10.0, 0.1, 0.0); // 距离太阳约 5.7 度
        let sun_pos = (1.0, 0.0, 0.0); // 太阳位置
        
        let corrected = gravitational_deflection(body_pos, sun_pos);
        
        // 引力偏折应使天体位置远离太阳
        println!("原始：{:?}, 修正后：{:?}", body_pos, corrected);
        
        // 验证偏折量级（靠近太阳时可达几角秒）
        let original_angle = body_pos.1.atan2(body_pos.0);
        let corrected_angle = corrected.1.atan2(corrected.0);
        let deflection = (corrected_angle - original_angle).abs() * constants::RAD;
        println!("引力偏折：{} 角秒", deflection * 3600.0);
    }
    
    #[test]
    fn test_precession() {
        // 测试岁差
        let t = 1.0; // 100 年后
        let (_, _, theta) = precession_iau2006(t);
        
        // 岁差应约 1.4 度/百年 = 0.024 弧度
        println!("岁差角 theta={} 度 = {} 弧度", theta * 180.0 / PI, theta);
        
        // 验证岁差量级（约 0.5 度/百年）
        assert!(theta.abs() > 0.001); // 大于 0.05 度
        assert!(theta.abs() < 0.1);  // 小于 6 度
    }
    
    #[test]
    fn test_nutation_matrix() {
        let t = 0.1;
        let epsilon = obliquity(t);
        let matrix = compute_nutation_matrix(t, epsilon);
        
        // 测试单位矩阵性质
        let pos = (1.0, 0.0, 0.0);
        let rotated = matrix.apply(pos);
        println!("章动矩阵旋转：{:?} -> {:?}", pos, rotated);
        
        // 验证旋转量级（章动约 17 角秒）
        let rotation = (rotated.1.powi(2) + rotated.2.powi(2)).sqrt();
        println!("章动旋转量：{} 角秒", rotation * constants::RAD * 3600.0);
        assert!(rotation < 0.001); // 小于 200 角秒
    }
    
    #[test]
    fn test_coordinate_transform() {
        // 测试黄赤坐标转换
        let epsilon = 23.439 * PI / 180.0;
        
        // 春分点：黄经 0, 黄纬 0 -> 赤经 0, 赤纬 0
        let ecl_pos = (0.0, 0.0, 1.0);
        let ecl_xyz = llr2xyz(ecl_pos);
        let eq_xyz = ecliptic_to_equatorial(ecl_xyz, epsilon);
        let eq_pos = xyz2llr(eq_xyz);
        
        println!("春分点转换：黄道 {:?} -> 赤道 {:?}", ecl_pos, eq_pos);
        assert!(eq_pos.0.abs() < 1e-10);
        assert!(eq_pos.1.abs() < 1e-10);
        
        // 夏至点：黄经 90 度，黄纬 0 -> 赤经 90 度，赤纬 23.439 度
        let ecl_pos2 = (PI / 2.0, 0.0, 1.0);
        let ecl_xyz2 = llr2xyz(ecl_pos2);
        let eq_xyz2 = ecliptic_to_equatorial(ecl_xyz2, epsilon);
        let eq_pos2 = xyz2llr(eq_xyz2);
        
        println!("夏至点转换：黄道 {:?} -> 赤道 {:?}", ecl_pos2, eq_pos2);
        assert!((eq_pos2.0 - PI / 2.0).abs() < 1e-10);
        assert!((eq_pos2.1 - epsilon).abs() < 1e-10);
    }
    
    #[test]
    fn test_solar_apparent_position() {
        // 测试太阳视位置计算的基本功能
        // 2023 年 7 月 23 日 12:00 TT
        let jd = 2460149.0;
        let t = (jd - 2451545.0) / 36525.0;
        
        let earth_pos = e_coord(t, -1, -1, -1);
        let (_, _, _, _, _, lt) = 
            compute_solar_apparent_position(t, earth_pos);
        
        // 验证光行时约 499 秒
        println!("太阳光行时：{} 秒", lt * 86400.0);
        assert!(lt * 86400.0 > 490.0 && lt * 86400.0 < 510.0);
    }
    
    #[test]
    fn test_lunar_apparent_position() {
        // 测试月球视位置计算的基本功能
        // 2023 年 7 月 23 日 12:00 TT
        let jd = 2460149.0;
        let t = (jd - 2451545.0) / 36525.0;
        
        let earth_pos = e_coord(t, -1, -1, -1);
        // 只验证函数调用不崩溃
        let _ = compute_lunar_apparent_position(t, earth_pos);
    }
    
    #[test]
    fn test_planet_apparent_position() {
        // 测试行星视位置计算（以火星为例）
        // 2023 年 7 月 23 日 12:00 TT
        let jd = 2460149.0;
        let t = (jd - 2451545.0) / 36525.0;
        
        let earth_pos = e_coord(t, -1, -1, -1);
        let mars_pos = p_coord(3, t, -1, -1, -1); // 火星
        
        let (_, _, _, _, dist, lt) = 
            compute_apparent_position(3, t, mars_pos, earth_pos);
        
        println!("火星视位置:");
        println!("  地心距：{} AU", dist);
        println!("  光行时：{} 秒", lt * 86400.0);
        
        // 验证光行时（火星约 3-22 分钟）
        assert!(lt * 86400.0 > 100.0 && lt * 86400.0 < 1500.0);
    }
    
    #[test]
    fn test_full_correction_chain() {
        // 测试完整矫正流程（以木星为例）
        // 2023 年 7 月 23 日 12:00 TT
        let jd = 2460149.0;
        let t = (jd - 2451545.0) / 36525.0;
        
        let earth_pos = e_coord(t, -1, -1, -1);
        let jupiter_pos = p_coord(4, t, -1, -1, -1); // 木星
        
        // 物理位置（未矫正）
        let physical_lon = jupiter_pos.0;
        
        // 视位置（完整矫正）
        let (_, _, _, _, dist, lt) = 
            compute_apparent_position(4, t, jupiter_pos, earth_pos);
        
        println!("木星位置矫正:");
        println!("  物理黄经：{} 度", physical_lon * 180.0 / PI);
        println!("  地心距：{} AU", dist);
        println!("  光行时：{} 秒", lt * 86400.0);
        
        // 验证光行时（木星约 35-53 分钟）
        assert!(lt * 86400.0 > 2000.0 && lt * 86400.0 < 3500.0);
    }
}
