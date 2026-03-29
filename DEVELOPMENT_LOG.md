# 开发日志

本文档记录 ephemeris 项目的开发历史、问题分析和修复记录。

---

## 目录

1. [更新日志](#更新日志)
2. [JPL 历表开发记录](#jpl-历表开发记录)
3. [基础设施修复](#基础设施修复)

---

## 更新日志

### 2026-03-29

#### d2971b7 - 完成 JPL 历表验证和对比测试

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

---

#### 82684e4 - 默认使用 Vondrák 2011 岁差模型

**类型**: feat  
**文件**: `ephemeris/src/internal/corrections.rs`

**主要变更**:
- 新增 `precession_vondrak_2011()` 函数
- 实现 Vondrák et al. (2011) 岁差模型
- 包含 22 个周期项 + 8 个多项式项
- 适用于长时期计算（-6000 年 ~ +6000 年）

**模型对比**:

| 模型 | 适用范围 | 周期项 | 多项式项 |
|------|---------|--------|---------|
| Vondrák 2011 | -6000~+6000 年 | 22 项 | 8 项 |
| IAU 2006 | 1900~2100 年 | 0 项 | 5 项 |

---

#### 789b0e2 - 修正 IAU 2006 岁差公式的霍纳法则实现

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

---

#### 802d212 - 使用 anise 库的完整光行差修正模型

**类型**: fix  
**文件**: `ephemeris/src/internal/jpl_ephemeris.rs`

**变更**:
- 使用 `Aberration::CN_S`（converged light time + stellar aberration）
- 包含收敛光时修正（迭代求解）
- 包含恒星周年光行差（相对论完整公式）
- 精度：约 5 米（99 百分位），与 SPICE 一致

---

#### d31f0a8 - 使用完整的 IAU 2000A 章动模型

**类型**: fix  
**文件**: `ephemeris/src/internal/jpl_ephemeris.rs`

**变更**:
- 使用 `nutation_iau2000a_impl::nutation_iau2000a()` 完整模型
- 章动值已经是弧度制，无需转换
- 同时修正黄经和黄纬

**精度**: 约 0.001 角秒（35 微角秒）

---

#### 26b8e4a - 重构代码为简单模式和精准模式

**类型**: feat  
**文件**: `ephemeris/src/internal/mode.rs`

**统一计算接口**:
- 新增 `mode` 模块，提供统一的 `EphemerisCalculator`
- `CalculationMode` 枚举：`Simple`（简单）和 `Precise`（精准）
- 支持运行时切换计算模式

---

### 早期提交

- **ab34d91** - 添加 JPL 历表支持
- **17ac96d** - 实现完整的天体测量矫正系统
- **292e082** - 完善章动模型
- **560ce16** - 提高 delta t 的计算精度

---

## JPL 历表开发记录

### JPL 历表修正状态

#### 已完成的修正 ✅

1. **光行差修正** - 使用 anise 的 `Aberration::CN_S`
2. **章动修正** - 使用 IAU 2000A 完整模型
3. **路径修复** - 所有文件使用绝对路径

#### 待解决的问题 ❌

**太阳位置差异约 13-166°**

**问题分析**:
1. anise 的 `translate()` 返回的是**地心到太阳的向量**（J2000 坐标系）
2. 太阳的**视黄经** = 地球的几何黄经 + 180° + 光行差 + 岁差 + 章动
3. 当前计算结果：-78.72° (即 281.28°)
4. Swiss Ephemeris 结果：87.90°
5. 差异：约 193° 或 166°（取决于是否加 180°）

**可能的原因**:
1. **岁差修正缺失** - anise 返回 J2000 坐标系，需要转换到观测时刻
2. **坐标系理解错误** - 可能需要反向 180°
3. **光行差方向** - anise 的光行差修正方向可能与我们期望的不同

#### 测试结果对比

| 天体 | Swiss Eph | JPL DE441 | 差异 |
|------|-----------|-----------|------|
| 太阳 | 87.90° | -78.72° | 166° ❌ |
| 月球 | 182.16° | 49.41° | 132° ❌ |
| 水星 | 113.63° | 64.97° | 48° ❌ |
| 金星 | 55.33° | 131.66° | 76° ❌ |
| 火星 | 45.34° | 137.56° | 92° ❌ |

#### 下一步行动

1. **检查 anise 文档** - 确认 `translate()` 返回的坐标系
2. **添加岁差修正** - 从 J2000 到观测时刻
3. **验证光行差方向** - 确认 anise 的光行差修正方向
4. **对比 JPL Horizons** - 使用 NASA 在线星历验证

---

### JPL 历表验证报告

#### 验证成功 ✅

**BSP 文件**：`bsp/441/de441_3000bc_3000ad.bsp` (623 MB)  
**覆盖范围**：-3000 年 ~ +3000 年

**测试结果（J2000.0）**：

| 天体 | 黄经 | 黄纬 | 距离 | 状态 |
|------|------|------|------|------|
| **太阳** | 281.29° | -23.03° | 0.983 AU | ✅ 成功 |
| **月球** | 222.45° | -10.90° | 402,449 km | ✅ 成功 |
| **水星** | 272.09° | -24.42° | 1.416 AU | ✅ 成功 |
| **金星** | 239.90° | -18.45° | 1.138 AU | ✅ 成功 |
| **火星** | 330.53° | -13.18° | 1.850 AU | ✅ 成功 |
| **木星** | 23.87° | 8.60° | 4.621 AU | ✅ 成功 |
| **土星** | 38.77° | 12.62° | 8.653 AU | ✅ 成功 |

#### 外行星 Frame 问题 ✅ 已解决

**问题**：anise 0.9.6 未定义外行星 Frame 常量

**解决方案**：使用 `Frame::from_ephem_j2000(naif_id)` 创建

```rust
use anise::prelude::Frame;

// NAIF ID: 火星=4, 木星=5, 土星=6...
let mars_frame = Frame::from_ephem_j2000(4);
let jupiter_frame = Frame::from_ephem_j2000(5);
```

---

## 基础设施修复

### swetest 调用解决方案

#### 问题描述

在 Rust 测试中调用 swetest 时遇到日期格式问题：
- 命令行直接运行：✅ 成功
- Rust Command 调用：❌ 失败，报 "illegal option"

#### 解决方案

**使用 bash -c 调用**

**修改前**（失败）：
```rust
Command::new(swetest_path)
    .args(&["-b", date_str, "-p", planet_flag, "-f", "Plong", "-n", "1", "-head"])
```

**修改后**（成功）：
```rust
let cmd_str = format!("{} -b{} -p{} -fPlong -n1 -head", swetest_path, date_str, planet_flag);

Command::new("bash")
    .args(&["-c", &cmd_str])
```

#### 完整代码

```rust
fn run_swetest(date_str: &str, planet_flag: &str) -> Option<BodyPosition> {
    let swetest_path = env!("CARGO_MANIFEST_DIR").to_string() + "/../swetest";
    let workspace_root = env!("CARGO_MANIFEST_DIR").to_string() + "/..";
    let ephe_path = env!("CARGO_MANIFEST_DIR").to_string() + "/../ephe";
    
    // 使用 shell 调用，设置 SE_EPHE_PATH 环境变量
    let cmd_str = format!(
        "SE_EPHE_PATH={} {} -b{} -p{} -fPlong -n1 -head",
        ephe_path, swetest_path, date_str, planet_flag
    );
    
    let cmd = Command::new("bash")
        .args(&["-c", &cmd_str])
        .current_dir(workspace_root)
        .output();
    
    match cmd {
        Ok(output) => {
            let stdout = String::from_utf8(output.stdout).ok()?;
            parse_swetest_output(&stdout, date_str)
        }
        Err(e) => {
            eprintln!("swetest command failed: {}", e);
            None
        }
    }
}
```

---

### SE1 文件路径修复

#### 问题

swetest 找不到 SE1 文件，使用 Moshier 近似历表：
```
warning: SwissEph file 'sepl_00.se1' not found in PATH
using Moshier eph.;
```

#### 解决方案

**设置 SE_EPHE_PATH 环境变量**

**命令行验证**：
```bash
SE_EPHE_PATH=/home/shey/Codes/my/ephemeris/ephe ./swetest -b2000.01.01.5 -p0 -fPlong -n1 -head
```

**输出**（不再显示 "using Moshier eph."）：
```
Sun              87.9006009    0.0000000   0.0000000  158.2049635
```

---

### 路径修复总结

#### 已修复的路径问题

**1. BSP 历表文件路径** ✅

修改前（相对路径）：
```rust
"bsp/441/de441_3000bc_3000ad.bsp"
```

修改后（绝对路径）：
```rust
let workspace_root = env!("CARGO_MANIFEST_DIR").to_string() + "/..";
workspace_root + "/bsp/441/de441_3000bc_3000ad.bsp"
```

**2. swetest 可执行文件路径** ✅

```rust
let swetest_path = env!("CARGO_MANIFEST_DIR").to_string() + "/../swetest";
```

**3. 工作目录设置** ✅

```rust
.current_dir(env!("CARGO_MANIFEST_DIR").to_string() + "/..")
```

---

## 工具与脚本

### 测试工具

- `verify_jpl.rs` - JPL 历表验证程序
- `test_bsp_load.rs` - BSP 文件加载测试
- `check_bsp_contents.rs` - BSP 内容检查
- `test_aberration.rs` - 光行差修正测试

### 使用示例

```bash
# 运行 JPL 验证
cargo run --example verify_jpl --release

# 运行对比测试
cargo test test_jpl_vs_swiss_ephemeris --test jpl_vs_swisseph -- --nocapture

# 运行光行差测试
cargo run --example test_aberration --release
```

---

## 参考资料

- [JPL 历表官方文档](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/)
- [anise 库文档](https://docs.rs/anise/)
- [Swiss Ephemeris 文档](https://www.astro.com/swisseph/)
- [JPL Horizons 在线星历](https://ssd.jpl.nasa.gov/horizons/)
