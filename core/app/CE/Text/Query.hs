-- | The `ce query` / `ce rules` console sentences (plan v2.32 step 5),
-- transcribed byte for byte from cli/src/query/console.rs and the
-- rules face's no-file line (cli/src/main_query.rs `show`).
module CE.Text.Query (catalogue) where

import CE.Text (Catalogue, table)

catalogue :: Catalogue
catalogue =
  table
    "no_rules_file\trules: no rules file — zero assertions\trules：没有规则文件——零断言\n\
    \error\terror at {}: {}\t错误 {}：{}\n\
    \degraded\tquery: degraded — {} (no judgment)\t查询：已降级——{}（未判决）\n\
    \assert_ok\tassert {}({}): ok\t断言 {}({})：通过\n\
    \assert_violations\tassert {}({}): {} violation(s)\t断言 {}({})：{} 条违规\n\
    \question\t?- {}: {} answer(s)\t?- {}：{} 个答案\n\
    \answer\t  {}\n\
    \proof\t{}{}({}){}\n\
    \by_fact\t (fact)\t（事实）\n\
    \by_clause\t (clause {})\t（第 {} 条子句）\n\
    \counts\t{} rule(s), {} fact row(s), {} derived, {} proof node(s){}\t{} 条规则、{} 行事实、{} 个派生元组、{} 个推导结点{}\n\
    \truncated\t; {} proof tree(s) past the budget left out\t；{} 棵推导树超预算未出"
