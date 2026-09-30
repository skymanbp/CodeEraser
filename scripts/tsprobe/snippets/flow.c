/* Flow sample (analysis-track §5.3, round 3): every statement,
   declaration and access shape the flow/1 lowering table names, in C11
   with the GNU spellings the tables read (and the C23 `[[fallthrough]]`
   GCC takes in gnu11). */
#include <setjmp.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>

struct point {
    int x;
    int y;
};

_Noreturn void die(const char *why);
void fail(int code) __attribute__((noreturn));
__attribute__((noreturn)) void halt(void);

static jmp_buf env;
extern int shared;

int branches(int a, int *out, struct point p, int count, ...) {
    int label;
    if (a > 0) {
        label = 1;
    } else if (a < 0) {
        label = -1;
    } else
        label = 0;
    int total = 0, i;
    for (i = 0; i < count; i++, total++) {
        if (i == 3) continue;
        if (i > 7) break;
        total += i;
    }
    for (int j = 0, k = 1; j < count; j++) {
        total += j * k;
    }
    while (total > 100) total--;
    do {
        total++;
    } while (total < 2);
    while (1) {
        if (total > 10) break;
        total = total + 1;
    }
    for (;;) {
        break;
    }
    while (true) {
        break;
    }
    if (total < 0) goto done;
    total += p.x;
done:
    *out = total;
    return label;
}

int apply(int (*cb)(int), int v) {
    return cb(v);
}

int switching(int kind) {
    int score = 0;
    switch (kind) {
    case 1:
        score = 1;
        [[fallthrough]];
    case 2:
        score += 2;
        break;
    case 3: {
        score = 3;
        break;
    }
    default:
        score = -1;
    }
    return score;
}

int accesses(struct point *p, int *items, int flag) {
    static int calls = 0;
    extern int shared;
    register int fast = 2;
    int x = 0;
    int y = fast, *ptr = &x, arr[3] = {0};
    calls++;
    x = x + 1;
    x += p->x;
    p->y = x;
    (*p).x = x;
    items[0] = x;
    *ptr = 5;
    --y;
    flag && (y = 1);
    flag || (y = 2);
    int pick = flag ? (x = 6) : (y = 7);
    {
        int x = 2;
        x++;
        y += x;
    }
    arr[1] = pick;
    return y + shared + arr[1];
}

int preprocessed(int v) {
    int out = v;
#if defined(DEBUG) && DEBUG > 1
    out = v * 2;
#elif defined(TRACE)
    out = v * 3;
#else
    out = v * 4;
#endif
#ifdef VERBOSE
    printf("%d\n", out);
#endif
    return out;
}

int dynamic(void) {
    int state = setjmp(env);
    if (state == 0) {
        longjmp(env, 1);
    }
    __asm__ volatile("nop");
    return state;
}

void fatal(int code) {
    if (code > 0) {
        exit(code);
    }
    if (code < 0) {
        abort();
    }
    die("zero");
    code = 0;
}
