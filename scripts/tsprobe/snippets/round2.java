package p;
public class Outer { class In { private void pm() {} protected void pr() {} void pk() {} public void pu() {} }
  void m() { class Local { void lm() {} } Runnable r = new Runnable() { public void run() {} }; }
  public static void main(String[] a) { outer: while (true) { continue outer; } }
}
