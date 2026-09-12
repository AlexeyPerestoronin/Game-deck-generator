// Raster HTML cards to JPEG. Chromium taints a canvas that draws SVG
// `<foreignObject>` (Firefox does not), so we paint the laid-out iframe DOM.
export async function htmlToJpegPages(html, widthMm, heightMm, dpi) {
  const cssPxPerMm = 96 / 25.4;
  const cssW = Math.max(1, widthMm * cssPxPerMm);
  const cssH = Math.max(1, heightMm * cssPxPerMm);
  const scale = dpi / 96;

  const iframe = document.createElement("iframe");
  iframe.setAttribute("sandbox", "allow-scripts allow-same-origin");
  iframe.setAttribute("style", [
    "position:fixed",
    "left:-12000px",
    "top:0",
    `width:${Math.ceil(cssW)}px`,
    `height:${Math.ceil(cssH)}px`,
    "border:0",
    "margin:0",
    "background:#fff",
  ].join(";"));
  document.body.appendChild(iframe);

  try {
    await new Promise((resolve, reject) => {
      iframe.addEventListener("load", () => resolve(), { once: true });
      iframe.addEventListener("error", () => reject(new Error("iframe failed to load")), { once: true });
      iframe.srcdoc = html;
    });
    await new Promise((resolve) => setTimeout(resolve, 0));

    const doc = iframe.contentDocument;
    const win = iframe.contentWindow;
    if (!doc || !win) {
      throw new Error("iframe has no document");
    }
    if (doc.fonts && doc.fonts.ready) {
      await doc.fonts.ready;
    }
    if (win.__fitHeaderNames) {
      try {
        await win.__fitHeaderNames;
      } catch (_err) {}
    }

    iframe.style.height = Math.max(cssH, doc.documentElement.scrollHeight) + "px";

    const cards = doc.querySelectorAll("article.card, article.card-back");
    const nodes = cards.length ? Array.from(cards) : [doc.body];
    const pages = [];
    for (const node of nodes) {
      pages.push(await rasterElement(node, cssW, cssH, scale));
    }
    return pages;
  } finally {
    iframe.remove();
  }
}

async function rasterElement(el, cssW, cssH, scale) {
  const w = Math.max(1, Math.round(cssW * scale));
  const h = Math.max(1, Math.round(cssH * scale));
  const canvas = document.createElement("canvas");
  canvas.width = w;
  canvas.height = h;
  const ctx = canvas.getContext("2d");
  ctx.fillStyle = "#ffffff";
  ctx.fillRect(0, 0, w, h);
  ctx.save();
  ctx.scale(scale, scale);
  ctx.beginPath();
  ctx.rect(0, 0, cssW, cssH);
  ctx.clip();
  const origin = el.getBoundingClientRect();
  paintNode(ctx, el, origin.left, origin.top);
  ctx.restore();

  const jpeg = await new Promise((resolve, reject) => {
    canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("JPEG encode failed"))), "image/jpeg", 0.92);
  });
  const buf = new Uint8Array(await jpeg.arrayBuffer());
  return { data: buf, width: w, height: h };
}

function paintNode(ctx, node, ox, oy) {
  if (node.nodeType === 3) {
    paintText(ctx, node, ox, oy);
    return;
  }
  if (node.nodeType === 1) {
    paintElement(ctx, node, ox, oy);
  }
}

function paintElement(ctx, el, ox, oy) {
  const tag = el.tagName;
  if (tag === "SCRIPT" || tag === "STYLE" || tag === "LINK" || tag === "META" || tag === "HEAD" || tag === "NOSCRIPT") {
    return;
  }
  const win = el.ownerDocument.defaultView;
  const style = win.getComputedStyle(el);
  if (style.display === "none" || style.visibility === "hidden") {
    return;
  }

  const rect = el.getBoundingClientRect();
  const x = rect.left - ox;
  const y = rect.top - oy;
  const width = rect.width;
  const height = rect.height;
  const alpha = Number(style.opacity);
  ctx.save();
  if (Number.isFinite(alpha) && alpha !== 1) {
    ctx.globalAlpha *= alpha;
  }

  if (width > 0 && height > 0) {
    const radius = borderRadii(style);
    if (!isTransparent(style.backgroundColor)) {
      pathRoundRect(ctx, x, y, width, height, radius);
      ctx.fillStyle = style.backgroundColor;
      ctx.fill();
    }
    paintReplacedImage(ctx, el, x, y, width, height);
    paintBorder(ctx, style, x, y, width, height, radius);
    paintListMarker(ctx, el, style, x, y);
    if (style.overflow !== "visible") {
      ctx.beginPath();
      pathRoundRect(ctx, x, y, width, height, radius);
      ctx.clip();
    }
  }

  if (tag !== "IMG" && tag !== "CANVAS" && tag !== "VIDEO" && tag !== "IFRAME") {
    for (const child of el.childNodes) {
      paintNode(ctx, child, ox, oy);
    }
  }
  ctx.restore();
}

function paintText(ctx, textNode, ox, oy) {
  const raw = textNode.nodeValue;
  if (!raw || !raw.trim()) {
    return;
  }
  const parent = textNode.parentElement;
  if (!parent) {
    return;
  }
  const win = textNode.ownerDocument.defaultView;
  const style = win.getComputedStyle(parent);
  ctx.fillStyle = style.color;
  ctx.font = `${style.fontStyle} ${style.fontWeight} ${style.fontSize} ${style.fontFamily}`;
  ctx.textAlign = "left";
  ctx.textBaseline = "middle";
  const transform = style.textTransform;
  const range = textNode.ownerDocument.createRange();
  for (let i = 0; i < raw.length; i++) {
    range.setStart(textNode, i);
    range.setEnd(textNode, i + 1);
    const r = range.getBoundingClientRect();
    if (r.width === 0 || r.height === 0) {
      continue;
    }
    let ch = raw[i];
    if (transform === "uppercase") {
      ch = ch.toUpperCase();
    } else if (transform === "lowercase") {
      ch = ch.toLowerCase();
    }
    ctx.fillText(ch, r.left - ox, r.top - oy + r.height / 2);
  }
}

function paintReplacedImage(ctx, el, x, y, width, height) {
  if (el.tagName !== "IMG") {
    return;
  }
  if (!el.complete || el.naturalWidth === 0) {
    return;
  }
  const src = el.currentSrc || el.src || "";
  if (!src.startsWith("data:") && !src.startsWith("blob:")) {
    try {
      if (new URL(src, document.baseURI).origin !== location.origin) {
        return;
      }
    } catch (_err) {
      return;
    }
  }
  ctx.drawImage(el, x, y, width, height);
}

function paintBorder(ctx, style, x, y, width, height, radius) {
  const top = parsePx(style.borderTopWidth);
  const right = parsePx(style.borderRightWidth);
  const bottom = parsePx(style.borderBottomWidth);
  const left = parsePx(style.borderLeftWidth);
  if (top === right && right === bottom && bottom === left) {
    if (top <= 0 || style.borderTopStyle === "none" || isTransparent(style.borderTopColor)) {
      return;
    }
    ctx.lineWidth = top;
    ctx.strokeStyle = style.borderTopColor;
    const inset = top / 2;
    const inner = radius.map((r) => Math.max(0, r - inset));
    pathRoundRect(ctx, x + inset, y + inset, Math.max(0, width - top), Math.max(0, height - top), inner);
    ctx.stroke();
    return;
  }
  strokeEdge(ctx, style.borderTopStyle, style.borderTopColor, top, x, y + top / 2, x + width, y + top / 2);
  strokeEdge(ctx, style.borderRightStyle, style.borderRightColor, right, x + width - right / 2, y, x + width - right / 2, y + height);
  strokeEdge(ctx, style.borderBottomStyle, style.borderBottomColor, bottom, x, y + height - bottom / 2, x + width, y + height - bottom / 2);
  strokeEdge(ctx, style.borderLeftStyle, style.borderLeftColor, left, x + left / 2, y, x + left / 2, y + height);
}

function strokeEdge(ctx, borderStyle, color, width, x0, y0, x1, y1) {
  if (width <= 0 || borderStyle === "none" || isTransparent(color)) {
    return;
  }
  ctx.lineWidth = width;
  ctx.strokeStyle = color;
  ctx.beginPath();
  ctx.moveTo(x0, y0);
  ctx.lineTo(x1, y1);
  ctx.stroke();
}

function paintListMarker(ctx, el, style, x, y) {
  if (el.tagName !== "LI") {
    return;
  }
  const type = style.listStyleType;
  if (!type || type === "none") {
    return;
  }
  const fs = parsePx(style.fontSize);
  const cx = x - fs * 0.55;
  const cy = y + fs * 0.55;
  const r = Math.max(1, fs * 0.18);
  ctx.fillStyle = style.color;
  ctx.beginPath();
  if (type === "square") {
    ctx.rect(cx - r, cy - r, r * 2, r * 2);
  } else {
    ctx.arc(cx, cy, r, 0, Math.PI * 2);
  }
  if (type === "circle") {
    ctx.lineWidth = Math.max(1, fs * 0.08);
    ctx.strokeStyle = style.color;
    ctx.stroke();
  } else {
    ctx.fill();
  }
}

function pathRoundRect(ctx, x, y, width, height, radius) {
  ctx.beginPath();
  if (typeof ctx.roundRect === "function") {
    ctx.roundRect(x, y, width, height, radius);
  } else {
    ctx.rect(x, y, width, height);
  }
}

function borderRadii(style) {
  return [
    parsePx(style.borderTopLeftRadius),
    parsePx(style.borderTopRightRadius),
    parsePx(style.borderBottomRightRadius),
    parsePx(style.borderBottomLeftRadius),
  ];
}

function parsePx(value) {
  const n = parseFloat(value);
  return Number.isFinite(n) ? n : 0;
}

function isTransparent(color) {
  return !color || color === "transparent" || color === "rgba(0, 0, 0, 0)";
}
