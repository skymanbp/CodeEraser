package com.example;
import java.util.List;
import static java.util.Collections.emptyList;
import java.util.*;
import com.example.sub.Other;
/** javadoc */
@Deprecated
public class Probe extends Base implements Iface {
    private int x;
    protected static final int K = 1;
    int pkgPrivate;
    public Probe() { this.x = 0; }
    // line
    /* block */
    public int get() { return x; }
    private static <T> T id(T t) { return t; }
    @Override public String toString() { return ""; }
    void loop(int n, List<String> list) {
        if (n > 1) {} else if (n == 0) {} else {}
        for (int i = 0; i < n; i++) { if (i == 1) continue; }
        for (String s : list) {}
        while (n-- > 0) { do { n++; } while (n < 0); }
        switch (n) { case 1: break; default: }
        int y = switch (n) { case 2 -> 1; default -> 0; };
        int t = n > 0 ? 1 : 0;
        try { get(); } catch (RuntimeException | Error e) { } finally { }
        outer: for (;;) { break outer; }
        Runnable r = () -> { get(); this.get(); };
        boolean b = n > 0 && n < 5 || n == 9;
        Probe.id(1); list.size(); super.hashCode();
        Object o = new Object() { public int hashCode() { return 1; } };
        char c = 'c'; String s2 = "str"; String tb = """
            text""";
    }
    public static void main(String[] args) { }
    interface Inner { void m(); }
    enum E { A, B }
    record R(int a) { R { } }
    @interface Anno {}
    class Nested { void nm() {} }
    static class SNested { }
}
