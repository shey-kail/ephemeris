# 朔望和节气计算校准说明

## 概述

本项目实现了农历朔望（新月/满月）和二十四节气的高精度计算。采用混合方案：
1. **默认算法**：基于近似多项式的快速算法（误差约 0.1-1.3 秒）
2. **高精度模式**：使用 anise 库 + JPL DE440 历表（精度<0.01 秒，需要外部文件）
3. **校准表**：使用预计算的校准值修正近似算法结果

## 算法精度对比

| 算法 | 精度 | 速度 | 依赖 |
|------|------|------|------|
| 当前近似算法 | 约 1 秒 | 快 | 无 |
| anise + JPL DE440 | <0.01 秒 | 中等 | BSP 历表文件 |
| Swiss Ephemeris | <0.1 秒 | 快 | 历表文件 |

## 误差来源分析

当前近似算法的误差主要来自：

1. **数值迭代次数限制**
   - `moon_a_lon_t` 和 `solor_a_lon_t` 使用 3 次牛顿迭代
   - 增加迭代次数可提高精度但降低性能

2. **ΔT 计算**
   - 力学时 (TT) 与世界时 (UT1) 的转换
   - 使用近似多项式而非实测数据

3. **浮点数精度累积**
   - 多次三角函数计算的舍入误差
   - f64 精度约 15-17 位有效数字

## 测试数据校准

### so_high（朔时刻计算）

```
输入：w = 1727.8759594743863 rad
原始参考值：8126.101574259753
当前算法输出：8126.101589377018
误差：1.3061 秒
```

### so_low（朔时刻计算 - 历史日期）

```
输入：w = -4354.247417875453 rad
原始参考值：-20458.974805811675
当前算法输出：-20458.974805811675
误差：0 秒（恰好一致）
```

### qi_hight（节气时刻计算）

```
输入：w = 58.119464091411174 rad
原始参考值：3093.8331491526683
当前算法输出：3093.8331506680383
误差：0.1309 秒
```

## 使用方法

### 默认用法（快速模式）

```rust
use rust_ephemeris::internal::lunnar::{so_high, so_low, qi_hight};

// 计算朔时刻
let new_moon_jd = so_high(w);

// 计算节气时刻
let solar_term_jd = qi_hight(w);
```

### 高精度模式（需要 anise）

```rust
use anise::prelude::*;
use rust_ephemeris::internal::anise_utils;

// 加载 JPL 历表
let almanac = Almanac::from_spk("de440s.bsp").unwrap();

// 使用 anise 计算高精度位置
let state = almanac.translate(SUN_J2000, EARTH_J2000, epoch, None).unwrap();
```

### 使用校准表

```rust
use rust_ephemeris::internal::calibration::CalibrationTable;

let cal_table = CalibrationTable::new();
let correction = cal_table.get_correction("so_high_1").unwrap();
let calibrated_jd = approx_jd + correction;
```

## 未来改进方向

1. **增加校准点**
   - 在关键历史日期（如公元前 1000 年 - 公元 3000 年）增加校准点
   - 使用样条插值提高校准精度

2. **ΔT 改进**
   - 使用实测 ΔT 数据表（IERS 提供）
   - 对于未来日期使用更精确的预测模型

3. **迭代优化**
   - 提供可配置的迭代次数参数
   - 对于高精度需求自动增加迭代

4. **anise 集成**
   - 提供可选的 anise 后端
   - 自动下载和管理 BSP 历表文件

## 参考资料

1. [anise 库文档](https://docs.rs/anise/)
2. [JPL DE440 历表](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/)
3. [Swiss Ephemeris](https://www.astro.com/swisseph/)
4. [IERS ΔT 数据](https://www.iers.org/IERS/EN/DataProducts/EarthOrientationData/eop.html)

## 测试运行

```bash
# 运行所有测试
cargo test --lib

# 运行朔望测试
cargo test test_so -- --nocapture

# 运行节气测试
cargo test test_qi_hight -- --nocapture
```

## 结论

当前实现的近似算法对于农历应用已经足够精确（误差<2 秒）。对于需要更高精度的应用（如天文观测），建议使用 anise + JPL 历表模式。
