/* a C header parsed under the C++ grammar */
#ifndef X_H
#define X_H
#include "y.h"
typedef struct node { struct node *next; int class; } node_t;
static inline int max2(int a, int b) { return a > b ? a : b; }
int add(int a, int b);
extern const char *name;
#endif
