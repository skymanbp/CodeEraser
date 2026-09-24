class K { void a(); public: void b(); protected: void c(); private: void d(); };
struct S { void e(); private: void f(); };
void K::b() { a(); this->a(); d(); }
static int file_local() { return 0; }
namespace outer { namespace inner { void deep() {} } }
int (*ptr)(int) = nullptr;
auto l2 = [](auto x) { return x; };
template<class T> void tf(T t) { if constexpr (sizeof(T) > 1) {} }
int main() { for (;;) { break; } return 0; }
