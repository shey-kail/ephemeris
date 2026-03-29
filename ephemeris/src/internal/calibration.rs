// 朔望和节气计算的校准模块
//
// 使用 anise 库生成的高精度参考值来校准近似算法
//
// 混合方案：
// 1. 日常使用：保持当前近似算法（速度快，误差约 1 秒）
// 2. 高精度需求：使用 anise + JPL 历表（精度高，需要外部文件）
// 3. 校准模式：使用预计算的校准表修正近似算法结果

use std::collections::HashMap;

/// 校准数据结构
#[derive(Debug, Clone)]
pub struct CalibrationData {
    pub w: f64,              // 输入参数（平黄经）
    pub expected_jd: f64,    // anise 计算的精确 JD
    pub approx_jd: f64,      // 近似算法计算的 JD
    pub correction: f64,     // 修正值（expected - approx）
}

/// 校准表生成器
pub struct CalibrationTable {
    data: HashMap<String, CalibrationData>,
}

impl CalibrationTable {
    pub fn new() -> Self {
        let mut data = HashMap::new();
        
        // 添加已知的校准点（这些值应该通过 anise + JPL DE440 计算得到）
        // 以下是示例数据，实际应该用 anise 重新计算
        
        // so_high 校准点
        data.insert("so_high_1".to_string(), CalibrationData {
            w: 1727.8759594743863,
            expected_jd: 8126.101574259753 + 2451545.0,  // 转换为绝对 JD
            approx_jd: 8126.101589377018 + 2451545.0,
            correction: -1.5117265e-5,  // 约 -1.3 秒
        });
        
        // so_low 校准点
        data.insert("so_low_1".to_string(), CalibrationData {
            w: -4354.247417875453,
            expected_jd: -20458.974805811675 + 2451545.0,
            approx_jd: -20458.97479069441 + 2451545.0,
            correction: -1.5117265e-5,  // 约 -1.3 秒
        });
        
        // qi_hight 校准点
        data.insert("qi_hight_1".to_string(), CalibrationData {
            w: 58.119464091411174,
            expected_jd: 3093.8331491526683 + 2451545.0,
            approx_jd: 3093.8331506680383 + 2451545.0,
            correction: -1.51537e-6,  // 约 -0.13 秒
        });
        
        Self { data }
    }
    
    /// 获取校准修正值
    pub fn get_correction(&self, key: &str) -> Option<f64> {
        self.data.get(key).map(|d| d.correction)
    }
    
    /// 使用线性插值获取校准修正值
    pub fn interpolate_correction(&self, w: f64, func_type: &str) -> f64 {
        // 找到最接近的两个校准点
        let key_pattern = format!("{}_{}", func_type, 1);
        
        if let Some(cal) = self.data.get(&key_pattern) {
            // 简单起见，直接使用最近的校准点
            // 实际应该实现线性插值或样条插值
            return cal.correction;
        }
        
        0.0 // 没有校准数据时返回 0
    }
}

/// 应用校准修正
pub fn apply_correction(jd: f64, correction: f64) -> f64 {
    jd + correction
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_calibration_table() {
        let table = CalibrationTable::new();
        
        // 测试 so_high 校准
        let corr = table.get_correction("so_high_1").unwrap();
        assert!(corr.abs() < 1e-4); // 修正值应该很小
        
        // 测试 so_low 校准
        let corr = table.get_correction("so_low_1").unwrap();
        assert!(corr.abs() < 1e-4);
        
        // 测试 qi_hight 校准
        let corr = table.get_correction("qi_hight_1").unwrap();
        assert!(corr.abs() < 1e-4);
    }
    
    #[test]
    fn test_apply_correction() {
        let jd = 2451545.0;
        let correction = -1.5e-5;
        let corrected = apply_correction(jd, correction);
        
        assert!((corrected - jd).abs() < 1e-4);
    }
}
