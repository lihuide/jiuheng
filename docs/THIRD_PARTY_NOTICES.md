# 第三方说明

本项目通过 Cargo.lock 固定 Rust 第三方依赖的版本和校验和。各依赖的许可证以其源码包内声明为准。

## kintaiyi 静态研究测试数据

`data/research/kintaiyi-counts-static-reference.json` 的阳/阴遁主客算参考值派生自 `kentang2017/kintaiyi` 固定提交 `626d8a5bd0f707fec2ed613c83d4b6611f56441d` 的 `src/kintaiyi/config.py` 字面量表，仅用于研究测试；未复制实现代码、未运行其 Python 模块、未作为运行时算法或古籍规则。

Copyright (c) 2023-2026 kentang2017. MIT License 全文保存在 [kintaiyi-MIT.txt](licenses/kintaiyi-MIT.txt)，须随派生测试数据与完整源码一同保留。详细来源、文件哈希与差异见 [太乙主客算开源对照报告](TAIYI_OPEN_SOURCE_COUNTS_CROSSCHECK.md)。

Windows 安装向导使用 Inno Setup。`packaging/ChineseSimplified.isl` 是 Inno Setup 6.5.0+ 的用户维护简体中文消息文件，维护者为 Zhenghan Yang (Kira)，原始项目：

https://github.com/kira-96/Inno-Setup-Chinese-Simplified-Translation

该文件保留了原始头部署名和来源说明。

## Solar Positioning 交叉参考实现

`divination-core` 的太阳视黄经实现根据公开的 NREL SPA 报告独立编写，并以 Klaus A. Brunner 的 MIT 许可 Java 实现进行系数及转录交叉核对。核对文件：`SPA.java`，blob SHA `7f2c3e6ffafce096a8038f0610119927cbbb8d35`。本项目不包含、复制或派生自 NREL 另行提供且带有分发限制的 C 源码。

Copyright (c) 2013, Klaus A. Brunner.

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## 天文模型资料

- 太阳位置与均时差过程：I. Reda, A. Andreas, *Solar Position Algorithm for Solar Radiation Applications*, NREL/TP-560-34302（2008 修订）；均时差采用该报告附录 A 的 A1/A2 式，并用附录示例的全精度参考值校验。
- ΔT 分段估算及历史不确定度：NASA Eclipse Web Site 的 Polynomial Expressions for Delta T 与 Uncertainty in Delta T。

这些公开资料用于算法重实现、测试向量和模型边界说明；它们不是古籍断法规则，也不会绕过古籍规则证据门禁。

## 现代干支历书校验资料

- 香港天文台《香港天文台年历 2024》用于干支日现代双锚点校验：<https://www.hko.gov.hk/tc/gts/astron2024/files/HKO_almanac_2024.pdf>
- 香港天文台“天干地支”说明页用于六十循环、十二时辰区间和日干配时干表的交叉校验：<https://www.hko.gov.hk/sc/gts/time/stemsandbranches.htm>
- 香港天文台节气资料页和《年历 2024》用于立春、雨水的现代分钟锚点校验：<https://www.hko.gov.hk/sc/gts/astronomy/Solar_Term.htm>、<https://www.hko.gov.hk/tc/gts/astron2024/files/HKO_almanac_2024.pdf>
- 香港天文台“今夕是何年？”用于明确官方正月初一换年与部分命理立春换年之间的定义差异：<https://www.hko.gov.hk/sc/education/astronomy-and-time/time-service/00506-what-year-is-it-today.html>

这些资料只校验历法标记的机械计算，不是古籍规则证据。子时换日的术式差异仍保持未决，计算结果不会直接参与裁决。

## GeoNames 地点数据

`data/locations.seed.json` 的行政地点名称、层级与 WGS‑84 中心坐标来自 GeoNames 中国数据转储：

https://download.geonames.org/export/dump/CN.zip

GeoNames 数据按 Creative Commons Attribution 4.0（CC BY 4.0）许可提供：

https://creativecommons.org/licenses/by/4.0/

Copyright / attribution: GeoNames — https://www.geonames.org/

生成后的每条记录保留 GeoNames ID、源记录修改日期、下载批次、许可和来源地址。坐标只作为对应行政要素的中心近似，不代表行政边界、街道门牌或 GPS 实测点。父级引用不完整、中文名称无法可靠恢复的记录会被生成器剔除，不以猜测补齐。

## regions-data 四级行政区划名称

`data/streets.seed.json` 的乡镇街道名称、代码与上下级关系由 `slightlee/regions-data` 的 2025-12-31 国家地名信息库整理快照确定性生成，快照提交为 `018603e0ef0124577807deb692a21040f9fcdd20`：

https://github.com/slightlee/regions-data/blob/018603e0ef0124577807deb692a21040f9fcdd20/data/processed/regions_20260304_173812.json

该仓库声明 MIT License；原始数据由国家地名信息库发布，仓库整理并提供 JSON。只有与 GeoNames 区县中心坐标唯一匹配的街道才进入可选地点库；街道名称不表示有街道中心坐标。各条记录同时保留街道行政代码和借用的 GeoNames ID。完整匹配口径见 `docs/LOCATION_STREET_CATALOG_AUDIT.md`。

Copyright (c) 2024 slightlee

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies
of the Software, and to permit persons to whom the Software is furnished to do
so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
