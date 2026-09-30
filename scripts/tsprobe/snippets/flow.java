// Flow sample (analysis-track §5.3, round 3): every statement,
// declaration and access shape the flow/1 lowering table names, in
// Java 21.
import java.io.BufferedReader;
import java.io.IOException;
import java.io.StringReader;
import java.util.List;
import java.util.function.IntUnaryOperator;

class Flow {
    private int total;
    private int[] items = new int[4];

    record Point(int x, int y) {}

    record Range(int lo, int hi) {
        Range {
            if (lo > hi) throw new IllegalArgumentException("range");
        }
    }

    Flow(int start) {
        this.total = start;
    }

    int receiver(Flow this, int by) {
        return total + by;
    }

    int branches(int a, int b, int... rest) {
        int label;
        if (a > 0) {
            label = 1;
        } else if (a < 0) {
            label = -1;
        } else label = 0;
        for (int i = 0, j = 1; i < b; i++, j++) {
            if (i == j) continue;
            if (i > 7) break;
            label += i;
        }
        for (int value : rest) {
            label -= value;
        }
        int n = 3;
        while (n > 0) n--;
        do {
            n++;
        } while (n < 2);
        while (true) {
            if (n > 10) break;
            n = n + 1;
        }
        outer:
        for (int p : rest) {
            for (int q : rest) {
                if (p == q) continue outer;
                if (p > q) break outer;
            }
        }
        found:
        {
            if (n > 5) break found;
            n = 0;
        }
        return label + n;
    }

    int switching(int kind, Object shape) {
        int score = 0;
        switch (kind) {
            case 1:
                score = 1;
            case 2, 3:
                score += 2;
                break;
            default:
                score = -1;
        }
        int arrow = switch (kind) {
            case 1 -> 10;
            case 2 -> {
                int doubled = kind * 2;
                yield doubled;
            }
            default -> throw new IllegalStateException("kind");
        };
        switch (shape) {
            case Point(int x, int y) when x > 0 -> score += x + y;
            case String s -> score += s.length();
            default -> score = 0;
        }
        if (shape instanceof Point p && p.x() > 0) {
            score += p.y();
        }
        return score + arrow;
    }

    String guarded(String text) {
        String line = "";
        try (BufferedReader reader = new BufferedReader(new StringReader(text));
                BufferedReader other = new BufferedReader(new StringReader(line))) {
            line = reader.readLine() + other.readLine();
            if (line == null) throw new IOException("empty");
        } catch (IOException | IllegalArgumentException err) {
            line = err.getMessage();
        } finally {
            total++;
        }
        try {
            line = line.trim();
        } catch (NullPointerException e) {
            return "";
        }
        return line;
    }

    int accesses(Point point, boolean flag) {
        int x = 0, y;
        x = x + 1;
        x += point.x();
        this.total = x;
        items[0] = x;
        y = flag && (x = 1) > 0 ? 2 : 3;
        y = flag || (x = 4) > 0 ? y : 5;
        var pick = flag ? (x = 6) : (y = 7);
        int[] local = {x, y};
        local[1] = pick;
        return local[0];
    }

    IntUnaryOperator closures(List<Integer> values) {
        int base = 10;
        int scale = 2;
        IntUnaryOperator add = d -> d + base;
        Runnable job = new Runnable() {
            private int seen = scale;

            @Override
            public void run() {
                System.out.println(base + seen);
            }
        };
        job.run();
        class Local {
            int twice(int v) {
                return v * scale;
            }
        }
        return v -> add.applyAsInt(v) + new Local().twice(values.get(0));
    }

    void fatal(int code) {
        if (code > 0) {
            System.exit(code);
        }
        throw new IllegalStateException("fatal");
    }
}
