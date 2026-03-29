// 使用 anise 库实现朔望和节气计算示例
//
// anise 库提供了：
// 1. 高精度历表计算（通过 SPK/BSP 文件）
// 2. 事件搜索框架（adaptive_step_scanner + brent_solver）
// 3. 天体位置计算

use anise::prelude::*;
use anise::constants::frames::{EARTH_J2000, SUN_J2000, MOON_J2000};
use anise::analysis::search::{adaptive_step_scanner, brent_solver};
use hifitime::{Epoch, TimeScale, Unit};

/// 计算太阳黄经（使用 anise 历表）
fn solar_longitude(almanac: &Almanac, epoch: Epoch) -> f64 {
    // 获取地心到太阳的位置向量
    let state = almanac
        .translate(SUN_J2000, EARTH_J2000, epoch, None)
        .unwrap();
    
    let x = state.x;
    let y = state.y;
    
    // 计算黄经（需要转换到黄道坐标系）
    // 这里简化为赤道坐标的经度
    y.atan2(x)
}

/// 计算月球黄经（使用 anise 历表）
fn lunar_longitude(almanac: &Almanac, epoch: Epoch) -> f64 {
    // 获取地心到月球的位置向量
    let state = almanac
        .translate(MOON_J2000, EARTH_J2000, epoch, None)
        .unwrap();
    
    let x = state.x;
    let y = state.y;
    
    y.atan2(x)
}

/// 计算朔（新月）时刻
/// 朔 = 太阳黄经 = 月球黄经
fn find_new_moon(almanac: &Almanac, start_epoch: Epoch) -> Epoch {
    // 定义搜索函数：当太阳黄经 - 月球黄经 = 0 时为朔
    let search_func = |epoch: Epoch| -> f64 {
        let sun_lon = solar_longitude(almanac, epoch);
        let moon_lon = lunar_longitude(almanac, epoch);
        sun_lon - moon_lon
    };
    
    // 使用自适应步长扫描找到符号变化区间
    let step = Unit::Hour * 1.0; // 1 小时步长
    let intervals = adaptive_step_scanner(start_epoch, step, 48.0, &search_func);
    
    // 使用 Brent 方法精确定位
    if let Some((t1, t2)) = intervals.first() {
        brent_solver(*t1, *t2, &search_func, 1e-9)
    } else {
        start_epoch // 未找到，返回起始时间
    }
}

/// 计算望（满月）时刻
/// 望 = 太阳黄经 - 月球黄经 = 180°
fn find_full_moon(almanac: &Almanac, start_epoch: Epoch) -> Epoch {
    // 定义搜索函数：当太阳黄经 - 月球黄经 = π 时为望
    let search_func = |epoch: Epoch| -> f64 {
        let sun_lon = solar_longitude(almanac, epoch);
        let moon_lon = lunar_longitude(almanac, epoch);
        let diff = (sun_lon - moon_lon).rem_euclid(2.0 * std::f64::consts::PI);
        diff - std::f64::consts::PI
    };
    
    let step = Unit::Hour * 1.0;
    let intervals = adaptive_step_scanner(start_epoch, step, 48.0, &search_func);
    
    if let Some((t1, t2)) = intervals.first() {
        brent_solver(*t1, *t2, &search_func, 1e-9)
    } else {
        start_epoch
    }
}

/// 计算节气时刻
/// 节气 = 太阳黄经为 15° 的整数倍
fn find_solar_term(almanac: &Almanac, start_epoch: Epoch, term_index: usize) -> Epoch {
    let target_longitude = term_index as f64 * 15.0 * std::f64::consts::PI / 180.0;
    
    // 定义搜索函数：当太阳黄经 = 目标黄经时为节气
    let search_func = |epoch: Epoch| -> f64 {
        let sun_lon = solar_longitude(almanac, epoch);
        (sun_lon - target_longitude).rem_euclid(2.0 * std::f64::consts::PI)
    };
    
    let step = Unit::Hour * 6.0; // 6 小时步长
    let intervals = adaptive_step_scanner(start_epoch, step, 15.0, &search_func);
    
    if let Some((t1, t2)) = intervals.first() {
        brent_solver(*t1, *t2, &search_func, 1e-9)
    } else {
        start_epoch
    }
}

fn main() {
    // 加载历表文件
    let almanac = Almanac::default();
    
    // 示例：计算 2023 年的朔望和节气
    let start = Epoch::from_gregorian_utc(2023, 1, 1, 0, 0, 0);
    
    // 计算朔（新月）
    let new_moon = find_new_moon(&almanac, start);
    println!("2023 年第一个朔：{}", new_moon);
    
    // 计算望（满月）
    let full_moon = find_full_moon(&almanac, start);
    println!("2023 年第一个望：{}", full_moon);
    
    // 计算立春（太阳黄经 315°）
    let lichun = find_solar_term(&almanac, start, 21);
    println!("2023 年立春：{}", lichun);
}

// 注意：
// 1. 实际使用需要加载 SPK/BSP 历表文件（如 de440s.bsp）
// 2. 需要正确处理坐标系转换（赤道→黄道）
// 3. 搜索范围和步长需要根据实际情况调整
