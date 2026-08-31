| | Without CodeEraser | With CodeEraser |
|---|---:|---:|
| writes refused before they reached disk | 0 | **2** |
| duplicate clone blocks left behind | 4 | **0** |
| duplicated doc segments | 1 | **0** |
| removals still owed | 1 | **0** |
| check score | 952 | **979** |

One seven-step task, two identical copies of the seed; the only variable is whether CodeEraser is in the loop — the write-time guard, the Stop audit, and, once the audit refuses, the eraser acting on its own plan. Both runs still end red — not on the same things.
