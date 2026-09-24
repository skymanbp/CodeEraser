#include <vector>
#include "local.hpp"
extern "C" { int c_linkage(int); }
extern "C" int c_one(int x) { return x; }
namespace ns {
  namespace { int hidden() { return 0; } }
  class Foo : public Base {
  public:
    Foo() : x_(0) {}
    virtual ~Foo();
    int get() const { return x_; }
    static int make(int a);
    template <typename T> T id(T t) { return t; }
    bool operator==(const Foo& o) const { return x_ == o.x_; }
  private:
    int x_;
  protected:
    void prot();
  };
  struct Bar { int y; void m() { this->y = 1; y = 2; ns::hidden(); } };
  int Foo::make(int a) { try { throw 1; } catch (int e) { return e; } catch (...) { return -1; } }
  auto lam = [](int a) -> int { return a; };
  void f(std::vector<int> vec, bool w) { for (auto v : vec) { if (v && w || !v) {} } }
  int g(int a) { return a ? 1 : 2; }
  using std::vector;
  using namespace std;
  int Bar::z = 3;
  void h(Bar obj, Bar* ptr) { g(1); Foo::make(2); obj.m(); ptr->m(); }
  enum class E { A, B };
  typedef int myint;
  using myint2 = int;
  template <typename T> struct Box { T v; };
  const char* s = "str"; char c = 'c'; auto r = R"(raw)";
}
