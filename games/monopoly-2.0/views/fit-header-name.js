(function () {
  const MIN_PT = 6;

  function availableBox(box) {
    const cs = getComputedStyle(box);
    return {
      width: box.clientWidth - parseFloat(cs.paddingLeft) - parseFloat(cs.paddingRight),
      height: box.clientHeight - parseFloat(cs.paddingTop) - parseFloat(cs.paddingBottom),
    };
  }

  function overflows(box, text) {
    const area = availableBox(box);
    return text.scrollWidth - area.width > 1 || text.scrollHeight - area.height > 1;
  }

  function fontSizePt(el) {
    return parseFloat(getComputedStyle(el).fontSize) * 72 / 96;
  }

  function fit(box) {
    const text = box.querySelector("span") || box.querySelector("ul") || box;
    box.style.fontSize = "";
    text.style.transform = "";
    if (!overflows(box, text)) {
      return;
    }

    let lo = MIN_PT;
    let hi = fontSizePt(box);
    if (hi <= MIN_PT) {
      box.style.fontSize = MIN_PT + "pt";
      return;
    }

    for (let i = 0; i < 16; i++) {
      const mid = (lo + hi) / 2;
      box.style.fontSize = mid + "pt";
      if (overflows(box, text)) {
        hi = mid;
      } else {
        lo = mid;
      }
    }
    box.style.fontSize = lo + "pt";
  }

  function fitAll() {
    document.querySelectorAll(".header .cell.name").forEach(fit);
    document.querySelectorAll(".effects-text").forEach(fit);
  }

  window.__fitHeaderNames = (
    document.fonts && document.fonts.ready
      ? document.fonts.ready
      : Promise.resolve()
  ).then(fitAll);
})();
