#!/usr/bin/env python3
"""
MRP 負庫存控制範例

展示如何從 ERP 系統通過 Python 設置 allow_negative_inventory 參數
"""

# 假設已安裝 mrp_engine Python 包
# pip install mrp-engine
from mrp_engine import MrpConfig

# ========== 場景 1: 批量生產 (MTS) - 不允許負庫存 ==========
print("場景 1: 批量生產 (Make-to-Stock)")
print("-" * 50)

config_mts = MrpConfig(
    component_id="PRODUCT-MTS-001",
    lead_time_days=5,
    procurement_type="Make",
    allow_negative_inventory=False  # ⚠️ 不允許負庫存
)

# 設置批量規則
config_mts.lot_sizing_rule = "FOQ"  # 固定批量
config_mts.fixed_lot_size = 100.0
config_mts.safety_stock = 50.0  # 安全庫存 50
config_mts.minimum_order_qty = 100.0

print(f"物料: {config_mts.component_id}")
print(f"生產方式: {config_mts.procurement_type}")
print(f"允許負庫存: {config_mts.allow_negative_inventory}")
print(f"安全庫存: {config_mts.safety_stock}")
print()
print("行為說明:")
print("- 當預計庫存 < 安全庫存時，立即產生計劃訂單")
print("- 適合：有實體庫存、需要保持安全庫存水位的物料")
print("- ERP 應用：成品、半成品、原物料")
print()


# ========== 場景 2: 按單生產 (MTO) - 允許負庫存 ==========
print("\n場景 2: 按單生產 (Make-to-Order)")
print("-" * 50)

config_mto = MrpConfig(
    component_id="PRODUCT-MTO-001",
    lead_time_days=10,
    procurement_type="Make",
    allow_negative_inventory=True  # ✅ 允許負庫存
)

# 設置批量規則
config_mto.lot_sizing_rule = "LotForLot"  # 批對批
config_mto.safety_stock = 0.0  # MTO 模式通常不需要安全庫存

print(f"物料: {config_mto.component_id}")
print(f"生產方式: {config_mto.procurement_type}")
print(f"允許負庫存: {config_mto.allow_negative_inventory}")
print(f"安全庫存: {config_mto.safety_stock}")
print()
print("行為說明:")
print("- 忽略安全庫存，只有當預計庫存 < 0 時才產生計劃訂單")
print("- 適合：無實體庫存、接單後才生產的物料")
print("- ERP 應用：客製化產品、工程專案、服務類物料")
print()


# ========== 場景 3: 虛擬件 - 允許負庫存 ==========
print("\n場景 3: 虛擬件 (Phantom)")
print("-" * 50)

config_phantom = MrpConfig(
    component_id="PHANTOM-SUBASSY-001",
    lead_time_days=0,  # 虛擬件無提前期
    procurement_type="Make",
    allow_negative_inventory=True  # ✅ 允許負庫存
)

config_phantom.lot_sizing_rule = "LotForLot"
config_phantom.safety_stock = 0.0

print(f"物料: {config_phantom.component_id}")
print(f"生產方式: {config_phantom.procurement_type}")
print(f"允許負庫存: {config_phantom.allow_negative_inventory}")
print(f"提前期: {config_phantom.lead_time_days} 天")
print()
print("行為說明:")
print("- 虛擬件不實際生產，只用於 BOM 結構")
print("- 允許負庫存避免不必要的計劃訂單")
print("- ERP 應用：虛設件、組裝階層、配置項")
print()


# ========== 從 ERP 數據庫動態載入配置範例 ==========
print("\n場景 4: 從 ERP 系統動態載入")
print("-" * 50)

# 模擬從 ERP 數據庫讀取物料主檔
erp_items = [
    {
        "component_id": "FG-BIKE-001",
        "lead_time": 7,
        "procurement": "Make",
        "mrp_type": "MTS",  # 從 ERP 讀取
        "safety_stock": 20.0,
    },
    {
        "component_id": "SG-SPECIAL-ORDER",
        "lead_time": 15,
        "procurement": "Make",
        "mrp_type": "MTO",  # 從 ERP 讀取
        "safety_stock": 0.0,
    },
]

configs = []
for item in erp_items:
    # 根據 ERP 的 MRP 類型決定是否允許負庫存
    allow_negative = (item["mrp_type"] == "MTO")

    config = MrpConfig(
        component_id=item["component_id"],
        lead_time_days=item["lead_time"],
        procurement_type=item["procurement"],
        allow_negative_inventory=allow_negative
    )
    config.safety_stock = item["safety_stock"]

    configs.append(config)

    print(f"✓ 載入 {item['component_id']}")
    print(f"  MRP類型: {item['mrp_type']}, 允許負庫存: {allow_negative}")

print()
print(f"共載入 {len(configs)} 個物料配置")


# ========== 總結 ==========
print("\n" + "=" * 50)
print("allow_negative_inventory 參數使用指南")
print("=" * 50)
print()
print("【不允許負庫存 (False)】- 預設值")
print("  適用場景:")
print("  • 批量生產 (MTS)")
print("  • 有實體庫存管理")
print("  • 需要維持安全庫存")
print("  • 原物料、半成品、成品")
print()
print("  ERP 對應:")
print("  • SAP: MRP Type = PD (MRP)")
print("  • Oracle: Planning Method = MRP Planning")
print()
print("【允許負庫存 (True)】")
print("  適用場景:")
print("  • 按單生產 (MTO)")
print("  • 按單採購 (PTO)")
print("  • 虛擬件 (Phantom)")
print("  • 服務類物料")
print("  • 無實體庫存")
print()
print("  ERP 對應:")
print("  • SAP: MRP Type = ND (No Planning)")
print("  • Oracle: Planning Method = MPS/MTO")
print()
