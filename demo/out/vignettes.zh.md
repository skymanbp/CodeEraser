**抄来的辅助函数，在文件存在之前就被拒。** 上表第 1 步单独拿出来。理由点名这段内容重复了哪块区域、以及怎样排序才能通过——所以这是一条可执行的拒绝，不是一张否决票。

```console
$ Write invoicer/discount.py
✗ ce：<work>/invoicer/discount.py 的内容与 1 处已索引区域重复：invoicer/money.py:1-18 (89 tokens)。请复用既有实现，而不是另写一份。若是在搬移？先删去源区域：探针以当前树为准校验，同一次写入随即通过。
```

**一条线，两张嘴。** `ce.toml` 给 `invoicer/**` 定下 `file_lines_fail = 40`。写入时守卫拒绝会越线的那次写入，`ce scan` 用同一个数给同一棵树评级——一处声明，钩子与 CI 同读。

```console
$ Write invoicer/invoice.py
✗ ce：这次写入会让 <work>/invoicer/invoice.py 达到 93 行，越过 40 行的硬预算（计划 §4.1）。请拆分文件，而不是继续让它长大。
$ ce scan .
FAIL invoicer/invoice.py:1 file-lines = 51（上限 40）[invoicer/invoice.py]
warn invoicer/report.py:1 file-lines = 35（上限 30）[invoicer/report.py]
已扫描 9 文件 / 19 函数 — 1 warn，1 fail -> FAIL（失败条件：hard_line）
```
