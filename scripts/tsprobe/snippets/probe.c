#include <stdio.h>
#include "local.h"
/* block comment */
// line comment
#define SQUARE(x) ((x)*(x))
#define LIMIT 10
static int helper(int a, int b) { return a && b || !a; }
int main(int argc, char **argv) {
    if (argc > 1) { puts("x"); } else if (argc == 0) { puts("y"); } else { puts("z"); }
    for (int i = 0; i < 3; i++) { if (i == 1) continue; if (i == 2) break; }
    while (argc--) { do { argc++; } while (argc < 0); }
    switch (argc) { case 1: puts("1"); break; case 2: default: puts("d"); }
    int t = argc > 0 ? 1 : 0;
#ifdef DEBUG
    t = 2;
#endif
    goto end;
end:
    return helper(argc, t) + SQUARE(t);
}
struct point { int x; int y; };
typedef struct { int a; } alias_t;
enum color { RED, GREEN };
union u { int i; float f; };
char c = 'a'; const char *s = "str\n";
void (*fp)(int) = 0;
int *ptr_fn(int n) { return 0; }
extern int external_decl(int);
