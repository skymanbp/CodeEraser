struct s { int (*cb)(int); };
void caller(struct s *p) { p->cb(1); (*p->cb)(2); }
int arr_fn(int a[]) { return a[0]; }
static void v(void) {}
int main(void) { char *x = "a" "b"; return 0; }
