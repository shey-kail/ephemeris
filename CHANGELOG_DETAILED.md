# 项目更新日志

本文档按时间顺序记录项目的重要更新，包含日期、版本号和详细说明。

---

## 2026-03-29

### d2971b7 - 完成 JPL 历表验证和对比测试

**类型**: feat  
**文件**: `ephemeris/tests/jpl_vs_swisseph.rs`, `ephemeris/examples/verify_jpl.rs`

**主要变更**:
1. 修复 JPL 历表路径为绝对路径（基于 CARGO_MANIFEST_DIR）
2. 添加外行星支持（使用质心 NAIF ID 4-9）
3. 创建对比测试框架（jpl_vs_swisseph.rs）
4. 修复 swetest 调用问题（使用 bash -c + SE_EPHE_PATH）

**测试功能**:
- 对比 JPL DE441 与 Swiss Ephemeris 的视黄经
- 支持所有行星（太阳、月球、水、金、火、木、土）
- 自动加载 SE1 历表文件

**使用方法**:
```bash
cargo test test_jpl_vs_swiss_ephemeris --test jpl_vs_swisseph -- --nocapture
```

---

### 82684e4 - 默认使用 Vondrák 2011 岁差模型

**类型**: feat  
**文件**: `ephemeris/src/internal/corrections.rs`

**主要变更**:
- 新增 `precession_vondrak_2011()` 函数
- 实现 Vondrák et al. (2011) 岁差模型
- 包含 22 个周期项 + 8 个多项式项
- 适用于长时期计算（-6000 年 ~ +6000 年）
- `apply_precession()` 默认使用 Vondrák 2011
- 保留 IAU 2006 为 `apply_precession_iau2006()`（短时期使用）

**模型对比**:
| 模型 | 适用范围 | 周期项 | 多项式项 |
|------|---------|--------|---------|
| Vondrák 2011 | -6000~+6000 年 | 22 项 | 8 项 |
| IAU 2006 | 1900~2100 年 | 0 项 | 5 项 |

---

### c19e0b2 - 添加岁差修正说明文档

**类型**: docs  
**文件**: `PRECESSION_FIX.md`

**说明**: 记录 IAU 2006 岁差公式霍纳法则系数修复过程。

---

### 789b0e2 - 修正 IAU 2006 岁差公式的霍纳法则实现

**类型**: fix  
**文件**: `ephemeris/src/internal/corrections.rs`

**问题**: 岁差公式系数顺序错误，导致计算结果不准确。

**修复**:
```rust
// 修复前（错误）
let zeta_arcsec = 2.650545 + 2306.083227 * t + 0.2988499 * t2 + ...

// 修复后（正确）
let zeta_arcsec = (((((-0.0000003173 * t - 0.000005971) * t + 0.01801828) * t + 0.2988499) * t + 2306.083227) * t + 2.650545) * t;
```

**参考**: Swiss Ephemeris `swephlib.c` `precess_1()` `SEMOD_PREC_IAU_2006`

---

### 802d212 - 使用 anise 库的完整光行差修正模型

**类型**: fix  
**文件**: `ephemeris/src/internal/jpl_ephemeris.rs`

**变更**:
- 使用 `Aberration::CN_S`（converged light time + stellar aberration）
- 包含收敛光时修正（迭代求解）
- 包含恒星周年光行差（相对论完整公式）
- 精度：约 5 米（99 百分位），与 SPICE 一致

**对比**:
- 简化公式：精度约 0.1 角秒
- anise CN_S: 精度约 0.001 角秒（100 倍提升）

---

### d31f0a8 - 使用完整的 IAU 2000A 章动模型

**类型**: fix  
**文件**: `ephemeris/src/internal/jpl_ephemeris.rs`

**变更**:
- 使用 `nutation_iau2000a_impl::nutation_iau2000a()` 完整模型
- 章动值已经是弧度制，无需转换
- 同时修正黄经和黄纬

**精度**: 约 0.001 角秒（35 微角秒）

---

### 781f29d - 精准模式支持行星视黄经计算

**类型**: feat  
**文件**: `ephemeris/src/internal/jpl_ephemeris.rs`, `ephemeris/src/internal/planet.rs`

**新增功能**:
- 新增 `planet` 模块，定义 `Planet` 枚举和 NAIF ID
- `JplEphemeris` 添加 `planet_position()` 计算行星位置
- `JplEphemeris` 添加 `planet_apparent_position()` 计算视位置
- 实现光行差修正和章动修正
- `EphemerisCalculator` 添加行星计算接口

**支持的行星**:
- 水星、金星、地球、火星、木星、土星、天王星、海王星、冥王星
- 月球、太阳

**外行星 Frame 解决方案**:
```rust
use anise::prelude::Frame;
let mars_frame = Frame::from_ephem_j2000(4);    // 火星质心
let jupiter_frame = Frame::from_ephem_j2000(5); // 木星质心
```

---

### 26b8e4a - 重构代码为简单模式和精准模式

**类型**: feat  
**文件**: `ephemeris/src/internal/mode.rs`, `ephemeris/src/internal/corrections.rs`

**统一计算接口**:
- 新增 `mode` 模块，提供统一的 `EphemerisCalculator`
- `CalculationMode` 枚举：`Simple`（简单）和 `Precise`（精准）
- 支持运行时切换计算模式

**简单模式（Simple Mode）**:
- 使用 Moshier 近似算法
- 无需外部历表文件
- 计算速度快（< 1 μs）
- 精度约 1-2 秒
- 适用于农历计算、一般应用

**精准模式（Precise Mode）**:
- 使用 JPL DE441 历表
- 需要 BSP 历表文件（约 500 MB）
- 计算速度较慢（~10 μs）
- 精度 < 0.1 秒
- 适用于高精度天文计算

**统一 API**:
- `solar_position()` - 太阳位置计算
- `lunar_position()` - 月球位置计算
- `find_new_moon()` - 朔时刻计算
- `find_full_moon()` - 望时刻计算
- `find_solar_term()` - 节气计算
- `planet_position()` - 行星位置计算（精准模式）

---

### ab34d91 - 添加 JPL 历表支持（todo #2）

**类型**: feat  
**文件**: `ephemeris/src/internal/jpl_ephemeris.rs`

**主要功能**:
- 新增 `jpl_ephemeris` 模块，封装 anise 库的 JPL 历表计算
- 支持 DE431 和 DE441 两种高精度历表
- 实现太阳和月球位置计算（精度 < 0.01°）
- 实现朔望时刻和节气时刻计算（精度 < 0.1 秒）
- 覆盖范围：公元前 13000 年 - 公元 17000 年

**API 接口**:
- `JplEphemeris::new()` - 创建历表计算器
- `solar_position()` - 计算太阳位置
- `lunar_position()` - 计算月球位置
- `find_new_moon()` - 计算朔时刻
- `find_full_moon()` - 计算望时刻
- `find_solar_term()` - 计算节气

---

### acd037f - 添加天体测量矫正和朔望计算的分析文档

**类型**: docs  
**文件**: `CALIBRATION.md`, `HISTORICAL_TEST_DATA.md`, `IMPROVEMENT_ATTEMPTS.md`, `SO_HIGH_ANALYSIS.md`, `SWISSEPH_ANALYSIS.md`

**说明**: 添加详细的天文算法分析文档，包括误差分析、改进尝试、Swiss Ephemeris 算法分析等。

---

### 17ac96d - 实现完整的天体测量矫正系统和历史年份朔望测试

**类型**: feat  
**文件**: `ephemeris/src/internal/corrections.rs`, `ephemeris/src/internal/calibration.rs`

**主要变更**:
- 新增 `corrections` 模块：实现光时、引力偏折、光行差、岁差、章动等矫正
- 新增 `calibration` 模块：提供朔望计算的校准框架
- 改进 `ephemeris` 模块：公开 `moon_a_lon`、`earth_lon` 等函数，添加二次插值求根算法
- 更新 `lunnar` 模块：改进 `so_high`/`so_low` 测试
- 新增历史年份朔望测试：覆盖公元前 3000 年到公元 3000 年（10 个测试点）

**精度说明**:
- `so_high` 误差约 1-2 秒（使用 `moon_a_lon_t2` 快速近似）
- `so_low` 误差约 0 秒（使用经验公式，针对特定时期优化）
- 历史年份测试所有点误差 < 0.1 秒，算法稳定性良好

---

## 早期提交

### 43be742 - 创建 pixi

**类型**: feat  
**日期**: 2026-03-29 之前

---

### 560ce16 - 提高 delta t 的计算精度

**类型**: feat  
**日期**: 2026-03-29 之前

---

### 292e082 - 完善章动模型

**类型**: feat  
**日期**: 2026-03-29 之前

---

## 总结

### 2026-03-29 主要工作内容

当天共完成 **10 次提交**，主要工作包括：

1. **JPL 历表支持** (ab34d91) - 添加完整的 JPL DE441 历表支持
2. **计算模式重构** (26b8e4a) - 统一简单模式和精准模式 API
3. **行星计算** (781f29d) - 支持所有行星的视黄经计算
4. **章动模型** (d31f0a8) - 使用完整的 IAU 2000A 模型
5. **光行差修正** (802d212) - 使用 anise 库的完整模型
6. **岁差修复** (789b0e2, c19e0b2, 82684e4) - 修复 IAU 2006 并添加 Vondrák 2011
7. **验证测试** (d2971b7) - 创建 JPL 与 Swiss Ephemeris 对比测试

### 核心文件变更

| 文件 | 变更次数 | 说明 |
|------|---------|------|
| `ephemeris/src/internal/jpl_ephemeris.rs` | 4 | JPL 历表、行星计算、光行差、章动 |
| `ephemeris/src/internal/corrections.rs` | 3 | 岁差模型、校准框架 |
| `ephemeris/src/internal/mode.rs` | 1 | 计算模式统一 |
| `ephemeris/src/internal/planet.rs` | 1 | 行星定义（新增） |
| `ephemeris/tests/jpl_vs_swisseph.rs` | 1 | 对比测试（新增） |

---

**文档生成时间**: 2026-03-29  
**最后更新 commit**: d2971b7
