// codeeraser.dev — the picture viewer. Every figure on the site that
// hangs a diagram or a GUI screenshot wraps it in a `.stage`; this file
// gives the stage a camera (drag to pan; wheel, pinch, keys and a
// button bar to zoom; full screen) and gives the screenshot strip its
// tabs. Progressive on purpose: with the file absent every stage is
// the plain picture it contains, the bar is never built, and the tab
// strip keeps the `hidden` it is written with — nothing dead is shown.
//
// The camera moves the <img> itself — its layout width and height,
// then a translate — rather than an SVG viewBox. One code path then
// zooms the vector diagrams and the raster screenshots alike, nothing
// is inlined into the page (an archify SVG carries a <style> whose
// --panel and --text would override the page's own tokens), and the
// engine re-rasterises a resized <img> of an SVG, so a zoomed diagram
// stays sharp. The camera is pure (no DOM) and exported under Node,
// which is how cli/tests/it/site_viewer.rs drives it.
//
// Editing this file? Bump ?v= on every page's <script>, for the same
// reason style.css states on its first lines.
(function (root) {
  "use strict";
  // 1x is the fitted picture and the floor: past it there is only
  // margin. 6x is the ceiling, where the smallest diagram label reads
  // like a heading. STEP is one button press or key; DOUBLE one
  // double-click; both geometric, so n in then n out lands on the start.
  var MAX = 6, STEP = 1.4, DOUBLE = 2;
  // Arrow keys move the window by this fraction of itself; a wheel
  // notch (deltaY 100) is an 11 % zoom; line and page wheel modes are
  // scaled to the pixel convention first so the gain means one thing.
  var PAN = 0.08, WHEEL = 0.0015, LINE_PX = 16, PAGE_PX = 400;
  // The bar's words, by the page's <html lang>: the pages carry one
  // language each (cli/tests/it/docs_lang.rs), so the bar must too.
  var TEXT = {
    en: { stage: "Picture: drag to pan, scroll to zoom once selected", zoomIn: "Zoom in", zoomOut: "Zoom out", reset: "Reset", full: "Full screen", hint: "Drag to pan · select, then scroll to zoom" },
    zh: { stage: "图片：拖动平移，点选后滚轮缩放", zoomIn: "放大", zoomOut: "缩小", reset: "复位", full: "全屏", hint: "拖动平移 · 点选后滚轮缩放" }
  };

  // one axis of the camera's clamp: centred while the picture fits
  // the stage along it, held to the edges once it does not
  function axis(pos, size, room) {
    return size <= room ? (room - size) / 2 : Math.min(0, Math.max(room - size, pos));
  }

  /// A picture of aspect `ratio` (width / height) inside a stage of
  /// w × h pixels: contain-fitted at scale 1, scaled about a chosen
  /// point after that, and never letting a picture edge cross into
  /// the stage once the picture is larger than it. `view()` answers
  /// the picture's box in stage pixels; every mutation returns it.
  function camera(ratio) {
    var stage = { w: 0, h: 0 };
    var s = 1, x = 0, y = 0;
    function base() {
      var w = stage.w, h = stage.w / ratio;
      if (h > stage.h) { h = stage.h; w = stage.h * ratio; }
      return { w: w, h: h };
    }
    function view() {
      var b = base(), w = b.w * s, h = b.h * s;
      x = axis(x, w, stage.w);
      y = axis(y, h, stage.h);
      return { x: x, y: y, w: w, h: h, s: s };
    }
    // the fraction of the picture under a stage point (0.5 for an empty stage)
    function at(v, px, py) {
      return v.w > 0 ? { fx: (px - v.x) / v.w, fy: (py - v.y) / v.h } : { fx: 0.5, fy: 0.5 };
    }
    // place the picture so its fraction f sits under the stage point (px, py)
    function place(f, px, py) {
      var b = base();
      x = px - f.fx * b.w * s;
      y = py - f.fy * b.h * s;
      return view();
    }
    return {
      view: view,
      resize: function (w, h) {
        var f = at(view(), stage.w / 2, stage.h / 2);
        stage.w = w;
        stage.h = h;
        return place(f, w / 2, h / 2);
      },
      zoomAt: function (factor, px, py) {
        var v = view(), next = Math.min(MAX, Math.max(1, s * factor));
        if (next === s) return v;
        var f = at(v, px, py);
        s = next;
        return place(f, px, py);
      },
      pan: function (dx, dy) { x += dx; y += dy; return view(); },
      reset: function () { s = 1; return view(); }
    };
  }

  // Under Node there is no document: hand the camera to the gate and stop.
  if (typeof module === "object" && module.exports) {
    module.exports = { camera: camera, TEXT: TEXT, MAX: MAX, STEP: STEP };
    return;
  }

  /// The pointers currently down on a stage — one is a drag, two are a
  /// pinch — and the finger spread at the last pinch sample.
  function fingers() {
    var live = {}, spread = 0;
    function ids() { return Object.keys(live); }
    function gap() {
      var k = ids();
      if (k.length < 2) return 0;
      var a = live[k[0]], b = live[k[1]];
      return Math.hypot(b.x - a.x, b.y - a.y);
    }
    return {
      down: function (e) { live[e.pointerId] = { x: e.clientX, y: e.clientY }; spread = gap(); },
      up: function (e) { delete live[e.pointerId]; spread = gap(); return ids().length; },
      // the move as a pan delta, or as a pinch (zoom factor about the midpoint)
      move: function (e) {
        var prev = live[e.pointerId];
        if (!prev) return null;
        live[e.pointerId] = { x: e.clientX, y: e.clientY };
        var k = ids();
        if (k.length < 2) return { dx: e.clientX - prev.x, dy: e.clientY - prev.y };
        var a = live[k[0]], b = live[k[1]], now = gap(), was = spread;
        spread = now;
        return { factor: was > 0 ? now / was : 1, mx: (a.x + b.x) / 2, my: (a.y + b.y) / 2 };
      }
    };
  }

  /// Stage-relative coordinates of a client point.
  function local(stage, cx, cy) {
    var r = stage.getBoundingClientRect();
    return { x: cx - r.left, y: cy - r.top };
  }

  /// Wire one stage's pointers and wheel to its camera. `paint` draws a
  /// view; `engaged` says whether the reader has taken hold of the
  /// figure (a pointer pressed or focus moved into it), because the
  /// wheel belongs to the page until then — a reader scrolling past a
  /// diagram keeps scrolling. Ctrl/Cmd + wheel already means zoom.
  function gestures(stage, cam, paint, engaged) {
    var f = fingers();
    stage.addEventListener("pointerdown", function (e) {
      if (e.pointerType === "mouse" && e.button !== 0) return;
      f.down(e);
      try { stage.setPointerCapture(e.pointerId); } catch (_) { /* pointer already gone */ }
      stage.classList.add("dragging");
      e.preventDefault();
      if (document.activeElement !== stage) stage.focus({ preventScroll: true });
    });
    stage.addEventListener("pointermove", function (e) {
      var m = f.move(e);
      if (!m) return;
      if (m.factor === undefined) { paint(cam.pan(m.dx, m.dy)); return; }
      var p = local(stage, m.mx, m.my);
      paint(cam.zoomAt(m.factor, p.x, p.y));
    });
    function up(e) {
      if (f.up(e) === 0) stage.classList.remove("dragging");
      if (stage.hasPointerCapture(e.pointerId)) stage.releasePointerCapture(e.pointerId);
    }
    stage.addEventListener("pointerup", up);
    stage.addEventListener("pointercancel", up);
    stage.addEventListener("dblclick", function (e) {
      var p = local(stage, e.clientX, e.clientY);
      e.preventDefault();
      paint(cam.zoomAt(e.shiftKey || e.altKey ? 1 / DOUBLE : DOUBLE, p.x, p.y));
    });
    stage.addEventListener("wheel", function (e) {
      if (!engaged() && !e.ctrlKey && !e.metaKey) return;
      e.preventDefault();
      var d = e.deltaY * (e.deltaMode === 1 ? LINE_PX : e.deltaMode === 2 ? PAGE_PX : 1);
      var p = local(stage, e.clientX, e.clientY);
      paint(cam.zoomAt(Math.pow(2, -d * WHEEL), p.x, p.y));
    }, { passive: false });
  }

  /// Keyboard on the focused stage: + − 0 and the arrows. Every other
  /// key keeps its meaning (Tab and Escape included).
  function keys(stage, cam, paint) {
    var move = { ArrowLeft: [-1, 0], ArrowRight: [1, 0], ArrowUp: [0, -1], ArrowDown: [0, 1] };
    stage.addEventListener("keydown", function (e) {
      if (e.ctrlKey || e.metaKey || e.altKey) return;
      var cx = stage.clientWidth / 2, cy = stage.clientHeight / 2;
      if (e.key === "+" || e.key === "=") paint(cam.zoomAt(STEP, cx, cy));
      else if (e.key === "-" || e.key === "_") paint(cam.zoomAt(1 / STEP, cx, cy));
      else if (e.key === "0") paint(cam.reset());
      else if (move[e.key]) paint(cam.pan(-move[e.key][0] * stage.clientWidth * PAN, -move[e.key][1] * stage.clientHeight * PAN));
      else return;
      e.preventDefault();
    });
  }

  /// The button bar under a stage, built here so the eight figures
  /// carry no copy of it; the full-screen button is left out where the
  /// API is missing rather than shown inert.
  function bar(figure, stage, cam, paint, t) {
    var el = document.createElement("div");
    el.className = "stage-bar";
    var full = document.fullscreenEnabled && figure.requestFullscreen ? '<button type="button" data-z="full">' + t.full + "</button>" : "";
    el.innerHTML = '<button type="button" data-z="in" aria-label="' + t.zoomIn + '">+</button>' +
      '<button type="button" data-z="out" aria-label="' + t.zoomOut + '">−</button>' +
      '<button type="button" data-z="reset">' + t.reset + "</button>" +
      '<span class="readout" data-z="readout" aria-live="polite">100%</span>' + full +
      '<span class="hint">' + t.hint + "</span>";
    el.addEventListener("click", function (e) {
      var b = e.target.closest("[data-z]");
      if (!b) return;
      var cx = stage.clientWidth / 2, cy = stage.clientHeight / 2, what = b.getAttribute("data-z");
      if (what === "in") paint(cam.zoomAt(STEP, cx, cy));
      else if (what === "out") paint(cam.zoomAt(1 / STEP, cx, cy));
      else if (what === "reset") paint(cam.reset());
      else if (what === "full" && document.fullscreenElement === figure) document.exitFullscreen();
      else if (what === "full") figure.requestFullscreen().catch(function () { /* refused: the page is unchanged */ });
    });
    stage.insertAdjacentElement("afterend", el);
    return el.querySelector('[data-z="readout"]');
  }

  /// One figure: the stage's <img> becomes the camera's picture (its
  /// width/height attributes are the aspect the stage keeps), and every
  /// resize — window, tab shown, full screen — re-fits through one
  /// observer.
  function setUp(figure, t) {
    var stage = figure.querySelector(".stage"), img = stage && stage.querySelector("img");
    var w = img && Number(img.getAttribute("width")), h = img && Number(img.getAttribute("height"));
    if (!(w > 0 && h > 0)) return;
    var cam = camera(w / h), engaged = false;
    stage.classList.add("live");
    stage.style.aspectRatio = w + " / " + h;
    stage.tabIndex = 0;
    stage.setAttribute("role", "group");
    stage.setAttribute("aria-label", t.stage);
    img.draggable = false;
    var readout = bar(figure, stage, cam, paint, t);
    function paint(v) {
      img.style.width = v.w + "px";
      img.style.height = v.h + "px";
      img.style.transform = "translate(" + v.x + "px, " + v.y + "px)";
      stage.classList.toggle("zoomed", v.s > 1);
      // fitted, a vertical swipe belongs to the page; zoomed, it pans the picture
      stage.style.touchAction = v.s > 1 ? "none" : "pan-y";
      var pct = Math.round(v.s * 100) + "%";
      if (readout.textContent !== pct) readout.textContent = pct;
    }
    new ResizeObserver(function () { paint(cam.resize(stage.clientWidth, stage.clientHeight)); }).observe(stage);
    figure.addEventListener("pointerdown", function () { engaged = true; });
    figure.addEventListener("focusin", function () { engaged = true; });
    document.addEventListener("pointerdown", function (e) { if (!figure.contains(e.target)) engaged = false; }, true);
    document.addEventListener("focusin", function (e) { if (!figure.contains(e.target)) engaged = false; }, true);
    gestures(stage, cam, paint, function () { return engaged; });
    keys(stage, cam, paint);
  }

  /// The screenshot strip: one panel shown at a time, the strip's
  /// buttons switching it, arrows moving along the strip (WAI-ARIA
  /// tabs). A hidden panel's stage has no size; showing it resizes it.
  function tabs(list) {
    var tab = [].slice.call(list.querySelectorAll('[role="tab"]'));
    var panel = tab.map(function (t) { return document.getElementById(t.getAttribute("aria-controls")); });
    function show(i, focus) {
      tab.forEach(function (t, j) {
        t.setAttribute("aria-selected", i === j ? "true" : "false");
        t.tabIndex = i === j ? 0 : -1;
        panel[j].hidden = i !== j;
      });
      if (focus) tab[i].focus();
    }
    list.addEventListener("click", function (e) {
      var t = e.target.closest('[role="tab"]');
      if (t) show(tab.indexOf(t), false);
    });
    list.addEventListener("keydown", function (e) {
      var i = tab.indexOf(document.activeElement), n = tab.length;
      if (i < 0) return;
      var to = { ArrowRight: (i + 1) % n, ArrowLeft: (i + n - 1) % n, Home: 0, End: n - 1 }[e.key];
      if (to === undefined) return;
      e.preventDefault();
      show(to, true);
    });
    list.hidden = false;
    show(0, false);
  }

  var t = TEXT[root.documentElement.lang] || TEXT.en;
  root.querySelectorAll(".stage").forEach(function (s) { setUp(s.closest("figure"), t); });
  root.querySelectorAll('[role="tablist"]').forEach(tabs);
})(typeof document === "object" ? document : null);
