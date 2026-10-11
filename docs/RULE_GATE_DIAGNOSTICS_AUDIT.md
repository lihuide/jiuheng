# 规则门禁逐项诊断审计（v0.68.0）

## 目的

v0.67.0 及更早版本已经以 SQLite 发布视图和写入触发器实施硬门禁，但开发者界面主要显示“字段、来源、复核、测试”四个总状态。任一总状态失败时，研究人员仍需人工逐表排查缺少哪一环。v0.68.0 增加只读诊断列表，让缺项可以直接定位，同时绝不把诊断结果作为放行依据。

## 稳定检查键

规则字段层：

- `fields.reviewed`、`fields.stability`、`fields.evidence_tier`
- `fields.original_text`、`fields.source_reference`
- `fields.program`、`fields.release_view`

主来源层：

- `source.primary_link`
- `source.verified_passage`
- `source.qualified_edition`
- `source.facsimile_bytes`
- `source.evidence_bundle`
- `source.release_view`

独立复核层：

- `review.bibliography`
- `review.transcription`
- `review.logic`
- `review.implementation`
- `review.no_rejection`
- `review.release_view`

测试层：

- `tests.enabled_cases`
- `tests.suite_hash`
- `tests.current_zero_failure_run`
- `tests.release_view`

每项包含稳定键、中文名称、通过布尔值与可审计说明。数量只用于界面展示，不形成百分比，不参与术式权重或古籍证据等级。

## 不改变的安全边界

- 是否允许执行仍只取 `rule_release_readiness` 四个数据库布尔字段。
- `rule_hits` 写入仍由数据库触发器阻止未放行 Rule ID。
- 运行时仍重新读取规则、编译确定性 JSON 程序并核对正文哈希。
- 诊断中的某个查询通过，不能替代完整主来源链、四类复核或当前零失败测试。
- 正式规则包为空时，界面仍显示 0 条规则，不生成占位结论、应期或行动建议。

## 自动验证

- 只有规则字段的测试夹具会明确列出缺少主来源、四类复核和测试链。
- 补齐来源及复核后，来源与复核检查全部通过，而测试检查继续保持失败。
- 补齐当前测试套件及零失败运行后，全部检查通过，且原有硬门禁同时允许执行。
- 修改来源校验和后，合格版本和影印字节匹配检查立即失败，原有主来源总门禁仍保持关闭。
