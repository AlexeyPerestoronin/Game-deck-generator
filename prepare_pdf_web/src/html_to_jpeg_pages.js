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

    const styleText = Array.from(doc.querySelectorAll("style"))
      .map((node) => node.textContent || "")
      .join("\n");
    const cards = doc.querySelectorAll("article.card, article.card-back");
    const nodes = cards.length ? Array.from(cards) : [doc.body];
    const pages = [];
    for (const node of nodes) {
      pages.push(await rasterElement(node, cssW, cssH, scale, styleText));
    }
    return pages;
  } finally {
    iframe.remove();
  }
}

async function rasterElement(el, cssW, cssH, scale, styleText) {
  const w = Math.max(1, Math.round(cssW * scale));
  const h = Math.max(1, Math.round(cssH * scale));
  const xhtml = new XMLSerializer().serializeToString(el);
  const svg =
    `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}">` +
    `<foreignObject width="100%" height="100%">` +
    `<div xmlns="http://www.w3.org/1999/xhtml" style="width:${cssW}px;height:${cssH}px;transform:scale(${scale});transform-origin:0 0;">` +
    `<style>${styleText}</style>` +
    xhtml +
    `</div></foreignObject></svg>`;
  const blob = new Blob([svg], { type: "image/svg+xml;charset=utf-8" });
  const url = URL.createObjectURL(blob);
  try {
    const img = new Image();
    await new Promise((resolve, reject) => {
      img.onload = () => resolve();
      img.onerror = () => reject(new Error("card raster failed"));
      img.src = url;
    });
    const canvas = document.createElement("canvas");
    canvas.width = w;
    canvas.height = h;
    const ctx = canvas.getContext("2d");
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, w, h);
    ctx.drawImage(img, 0, 0);
    const jpeg = await new Promise((resolve, reject) => {
      canvas.toBlob((b) => (b ? resolve(b) : reject(new Error("JPEG encode failed"))), "image/jpeg", 0.92);
    });
    const buf = new Uint8Array(await jpeg.arrayBuffer());
    return { data: buf, width: w, height: h };
  } finally {
    URL.revokeObjectURL(url);
  }
}
