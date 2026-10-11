# 七政四余：五星 JPL 长期近似向量研究审计

状态：**仅现代天文研究候选；不是七政四余完整星位，不可判宿度、宫位或吉凶。**

## 本轮实际实现

- 独立 Rust 模块读取本地常量，不联网、不下载星历；常量来自 NASA/JPL Solar System Dynamics《[Approximate Positions of the Planets](https://ssd.jpl.nasa.gov/planets/approx_pos.html)》Table 2a、2b。
- 对辰星、太白、荧惑、岁星、填星和地月质心分别计算开普勒轨道的 J2000 平黄道日心直角坐标；再保存“行星减地月质心”的几何向量、黄经、黄纬和距离。
- 木星、土星的长期平近点角额外项已按 Table 2b 加入；失效日期、非有限数值、轨道参数异常和迭代不收敛均报错。
- 新建占问的出生时间审计 v8 保存完整五星向量、计算版本和原始 `JD(TT)`；历史开发者模式直接读取已保存的候选，重放时逐字段重新计算校验。v7 及更旧快照仍可读取，但不使用新版公式静默回填。
- v0.63.0 又编码了同一 JPL 页面 Table 1 的较短年代参数，并在 1800–2049 的保守重叠区间比较 Table 1 与 Table 2 的五星地月质心相对方向。四个固定样本（共 20 个行星/时刻组合）在 1800 年起点、J2000、JD 2460000.5 和 2049 年末的最大方向差分别约为 0.273°、0.189°、0.087°、0.065°，均出现在土星；结果与数据见 `data/research/qizheng-five-planets-jpl-fit-overlap.json`。这些数值是**同一来源的两套拟合公式之间的差异**，既不是实际误差，也不是可信精度上限。

## 重要限制

JPL 这套系数供低精度用途，网页列出的名义误差针对**日心**坐标，不能当成这里“行星减地月质心”黄经的保证误差。尤其接近地球的视线几何关系或宿界时，不能凭这组数值给出宿位。软件还暂以 TT 代替 JPL 的 TDB，未做地月质心到地心、光行时、光行差、岁差章动到历元当日或视差修正。它更不是日月位置或四余的历史定义。上述任一项都不应被 UI 省略。

JPL [Horizons API](https://ssd-api.jpl.nasa.gov/doc/horizons.html) 可提供同坐标系参考向量，但本环境的实时接口连接超时，未取得或伪造任何独立向量。新增双表检查只能排查大偏差和回归，不能代替高精度星历对拍；**不声称达到术数实用级定位精度**。未做任一中国古籍规则的影印、异文、逻辑、实现四类复核；Rule ID、信号、应期、行动建议继续为零。

研究元数据见 `data/research/qizheng-five-planets-jpl-approx.json` 与 `data/research/qizheng-five-planets-jpl-fit-overlap.json`，代码见 `crates/divination-core/src/planetary_jpl_approx.rs`。
