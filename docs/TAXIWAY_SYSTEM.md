# 滑行道系统设计文档

## 概述

滑行道系统是 ATC 地面管制的核心功能，负责指挥飞机在机场地面安全、高效地移动。

## 数据结构

### 1. 滑行道（Taxiway）

```rust
pub struct Taxiway {
    pub name: String,           // 滑行道名称（如 A, B, C, A1, A2）
    pub taxiway_type: TaxiwayType,
    pub connects_to: Vec<String>, // 连接的其他滑行道
}

pub enum TaxiwayType {
    Main,      // 主滑行道（平行于跑道）
    Parallel,  // 平行滑行道
    Rapid,     // 快速脱离道（高速脱离跑道）
    Apron,     // 停机坪滑行道
}
```

### 2. 停机位（Gate）

```rust
pub struct Gate {
    pub number: String,         // 停机位号（如 A01, B12, 201）
    pub terminal: String,       // 航站楼（T1, T2, T3）
    pub aircraft_type: String,  // 适用机型（Heavy, Medium, Light）
    pub taxiway_access: Vec<String>, // 可达滑行道
}
```

### 3. 滑行路线（TaxiRoute）

```rust
pub struct TaxiRoute {
    pub from: String,           // 起点（跑道或停机位）
    pub to: String,             // 终点（跑道或停机位）
    pub route: Vec<String>,     // 滑行道序列
    pub distance: u32,          // 距离（米）
    pub hotspots: Vec<String>,  // 热点区域
}
```

### 4. 热点区域（Hotspot）

热点区域是机场地面容易发生冲突、需要特别注意的地方。

```rust
pub struct Hotspot {
    pub id: String,             // 热点编号（如 HS1, HS2）
    pub location: String,       // 位置描述
    pub description: String,    // 详细说明
    pub warning: String,        // 警告信息
}
```

## 滑行指令示例

### 中文标准用语

**离场滑行（停机位 → 跑道）：**
```
"国航101，地面，经滑行道A、B滑行至跑道01号等待点，注意热点区域HS1"
"CA101, ground, taxi via A, B to runway 01 holding point, caution hotspot HS1"
```

**进场滑行（跑道 → 停机位）：**
```
"东航202，塔台，经快速脱离道R3、滑行道C、D滑行至T2航站楼203号停机位"
"MU202, tower, vacate via rapid exit R3, taxi via C, D to Terminal 2 gate 203"
```

**穿越跑道：**
```
"南航303，地面，经滑行道E穿越跑道18号，然后经F滑行至36号等待点"
"CZ303, ground, taxi via E, cross runway 18, then via F to runway 36 holding point"
```

### 英文标准用语（ICAO）

**Departure Taxi:**
```
"Air China 101, ground, taxi via Alpha, Bravo to runway zero one holding point, caution hotspot Hotel Sierra One"
```

**Arrival Taxi:**
```
"China Eastern 202, tower, vacate via Romeo Three, taxi via Charlie, Delta to Terminal Two gate Two Zero Three"
```

**Runway Crossing:**
```
"China Southern 303, ground, taxi via Echo, cross runway one eight, then via Foxtrot to runway three six holding point"
```

## 北京首都机场（ZBAA）滑行道示例

### 滑行道布局

```
主滑行道：
- A: 平行于 01/19 跑道东侧
- B: 平行于 18L/36R 跑道西侧
- C: 平行于 18R/36L 跑道东侧

连接滑行道：
- A1, A2, A3: 连接 A 和停机坪
- B1, B2, B3: 连接 B 和停机坪
- C1, C2, C3: 连接 C 和停机坪

快速脱离道：
- R1, R2, R3: 01 跑道快速脱离
- R4, R5, R6: 36R 跑道快速脱离
```

### 常用滑行路线

**T3 → 跑道 01：**
```
路线：停机位 → A3 → A → 01 等待点
距离：约 2500 米
热点：HS1（A 与 B 交叉点）
```

**跑道 36R → T2：**
```
路线：36R → R5 → B → B2 → 停机位
距离：约 1800 米
热点：HS2（B 与 C 交叉点）
```

## 热点区域管理

### 常见热点类型

1. **跑道交叉点**
   - 描述：滑行道穿越活动跑道
   - 警告："Hold short of runway XX unless cleared to cross"

2. **滑行道交叉点**
   - 描述：多条滑行道交汇
   - 警告："Give way to traffic from left/right"

3. **停机坪入口**
   - 描述：主滑行道进入停机坪
   - 警告："Reduce speed, watch for ground vehicles"

4. **施工区域**
   - 描述：临时施工或维护区域
   - 警告："Construction area, follow marshaller instructions"

## LLM 集成

### System Prompt 增强

在 LLM 的 system prompt 中添加滑行道信息：

```markdown
**滑行道系统：**
- 主滑行道 A（平行于 01/19）→ 连接：A1、A2、A3
- 主滑行道 B（平行于 18L/36R）→ 连接：B1、B2、B3
- 快速脱离道 R1-R6

**常用滑行路线：**
- T3 → 跑道 01：经 A3、A 至 01 等��点（约 2500 米）
- 跑道 36R → T2：经 R5、B、B2 至停机位（约 1800 米）

**热点区域：**
- HS1（A 与 B 交叉点）：注意交叉交通
- HS2（B 与 C 交叉点）：减速慢行

**滑行指令要求：**
1. 必须包含完整的滑行道序列
2. 必须说明目的地（跑道���待点或停机位）
3. 遇到热点区域必须提醒
4. 穿越跑道必须明确许可
```

### 指令生成逻辑

```rust
// 根据飞行阶段自动选择滑行路线
match flight_phase {
    FlightPhase::Parked => {
        // 离场滑行：停机位 → 跑道
        if let Some(route) = airport.get_taxi_route(&current_gate, &departure_runway) {
            let instruction = airport.format_taxi_instruction(route, language);
            // "经滑行道 A3、A 滑行至跑道 01 等待点，注意热点区域 HS1"
        }
    }
    FlightPhase::Landing => {
        // 进场滑行：跑道 → 停机位
        if let Some(route) = airport.get_taxi_route(&arrival_runway, &assigned_gate) {
            let instruction = airport.format_taxi_instruction(route, language);
            // "经快速脱离道 R5、滑行道 B、B2 滑行至 T2 航站楼 203 号停机位"
        }
    }
    _ => {}
}
```

## 数据来源

### 1. X-Plane apt.dat

X-Plane 的 `apt.dat` 文件包含详细的滑行道数据：

```
1302 taxiway_sign 40.080100 116.584600 180.00 5 {@Y,^r}
1204 taxiway A
1204 taxiway B
1204 taxiway C
```

### 2. MSFS BGL

MSFS 的 BGL 文件包含机场布局，需要解析工具。

### 3. Navigraph

Navigraph 提供专业的机场图表和滑行道数据。

### 4. 手动维护

对于常用机场，可以手动维护关键滑行路线和热点区域。

## 实现优先级

### Phase 1：基础滑行指令（当前）
- ✅ 数据结构定义
- ✅ 滑行路线查询
- ✅ 指令格式化
- ⏳ 硬编码 4 个主要机场的滑行道数据

### Phase 2：智能路线规划
- ⏳ 根据飞机位置自动选择最优路线
- ⏳ 避开拥堵区域
- ⏳ 考虑飞机类型（大型机避开窄滑行道）

### Phase 3：实时冲突检测
- ⏳ 检测滑行道占用情况
- ⏳ 避免冲突
- ⏳ 动态调整路线

### Phase 4：全量数据集成
- ⏳ 解析 X-Plane apt.dat
- ⏳ 集成 Navigraph 数据
- ⏳ 支持全球机场

## 测试场景

### 场景 1：简单离场
```
飞机：CA101，停机位 T3-A01
目标：跑道 01
期望指令："国航101，地面，经滑行道 A3、A 滑行至跑道 01 号等待点"
```

### 场景 2：穿越跑道
```
飞机：MU202，停机位 T2-B12
目标：跑道 36L
期望指令："东航202，地面，经滑行道 B2、B 穿越跑道 36R，然后经 C 滑行至跑道 36L 等待点"
```

### 场景 3：热点区域
```
飞机：CZ303，停机位 T1-C05
目标：跑道 01
期望指令："南航303，地面，经滑行道 C1、C、A 滑行至跑道 01 号等待点，注意热点区域 HS1"
```

## 参考资料

- ICAO Doc 9157 - Aerodrome Design Manual
- FAA AC 150/5340-18 - Standards for Airport Sign Systems
- ICAO Annex 14 - Aerodromes
- 中国民航局《民用机场飞行区技术标准》(MH 5001-2021)
