// 检查 BSP 文件包含的天体
use anise::prelude::*;

fn main() {
    println!("=== 检查 BSP 文件内容 ===\n");
    
    let bsp_path = "bsp/441/de441_3000bc_3000ad.bsp";
    
    match SPK::load(bsp_path) {
        Ok(spk) => {
            println!("✓ 成功加载：{}", bsp_path);
            
            // 尝试读取 DAF 摘要
            println!("\n尝试读取摘要...");
            for i in 0..20 {
                match spk.daf_summary(Some(i)) {
                    Ok(summary) => {
                        println!("  {}: center={}, target={}", i, summary.center, summary.target);
                    }
                    Err(_) => {
                        println!("  {}: 无数据", i);
                        break;
                    }
                }
            }
        }
        Err(e) => {
            println!("✗ 加载失败：{}", e);
        }
    }
}
