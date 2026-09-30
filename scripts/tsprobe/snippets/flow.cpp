// Flow sample (analysis-track §5.3, round 3): every statement,
// declaration and access shape the flow/1 lowering table names, in C++20.
#include <cstdlib>
#include <exception>
#include <map>
#include <stdexcept>
#include <string>
#include <vector>

[[noreturn]] void quit_now(int code);
__attribute__((noreturn)) void halt(int code);
extern int shared_count;

struct Point {
    int x;
    int y;
    int shift(int by) {
        this->x += by;
        return x;
    }
};

int branches(int a, const std::vector<int>& items, int scale = 2) {
    int label = 0;
    if (a > 0) {
        label = 1;
    } else if (a < 0) {
        label = -1;
    }
    if (int v = a * 2; v > 4) {
        label += v;
    }
    for (int i = 0; i < a; ++i) {
        if (i == 3) continue;
        if (i > 7) break;
        label += i;
    }
    for (const auto& item : items) {
        label += item;
    }
    for (auto [first, second] : std::map<int, int>{}) {
        label += first + second;
    }
    while (true) {
        if (label > 10) break;
        label = label + 1;
    }
    do {
        label++;
    } while (label < 2);
    return label * scale;
}

int switching(int kind) {
    int score = 0;
    switch (int k = kind * 2; k) {
    case 1:
        score = 1;
        [[fallthrough]];
    case 2:
        score += 2;
        break;
    default:
        score = -1;
    }
    return score;
}

int guarded(const std::string& text) {
    int value = 0;
    try {
        value = std::stoi(text);
        if (value < 0) throw std::invalid_argument("negative");
    } catch (const std::invalid_argument& err) {
        value = -1;
    } catch (...) {
        throw;
    }
    return value;
}

int accesses(Point& p, int* items, bool flag) {
    int x = 0;
    int& ref = x;
    auto& alias = p;
    auto&& fwd = x;
    int* ptr = &x;
    ref = 3;
    alias.y = x;
    *ptr = 5;
    items[0] = fwd;
    auto [px, py] = p;
    flag && (x = 1);
    flag || (x = 4);
    flag and (x = 2);
    flag or (x = 3);
    int pick = flag ? (x = 6) : 7;
    static int calls = 0;
    ++calls;
    return px + py + pick;
}

int closures(std::vector<int>& values) {
    int base = 10;
    int scale = 2;
    auto add = [&](int d) {
        scale += d;
        return d + base;
    };
    auto copy = [base](int d) { return d * base; };
    struct Local {
        int twice(int v) { return v * 2; }
    };
    return add(values[0]) + copy(1) + Local{}.twice(scale);
}

int family(int n) {
    int grid[3] = {0, 1, 2};
    int (*pick)(int) = nullptr;
    int total = 0;
    for (int i = 0, j = n; i < j; i++, j--) {
        total += grid[i % 3];
    }
#if defined(FLOW_FAST)
    total *= 2;
#elif defined(FLOW_SLOW)
    total /= 2;
#else
    total += 1;
#endif
#ifdef FLOW_TRACE
    total -= 1;
#endif
    if (total > 100) goto done;
    total = pick ? pick(total) : total;
    __asm__ volatile("");
done:
    return total + shared_count;
}

void fatal(int code) {
    if (code > 0) {
        std::exit(code);
    }
    if (code < 0) {
        std::terminate();
    }
    quit_now(code);
    code = 0;
}

struct Holder {
    Holder(int a, int b) : a_(a), b_(b + 1) {}
    int a_, b_;
};

void vexing(int data, int size) {
    wrapper w(data, sizeof(size));
}
