// Save what a dither element is showing as a PNG, with `scale` pixels per cell. The
// result has the cells at full size, so it stays crisp however it is used.

export async function savePng(element, name = 'dither.png', scale = 3) {
  const source = element.querySelector('canvas');
  const out = new OffscreenCanvas(source.width * scale, source.height * scale);
  const painter = out.getContext('2d');
  painter.imageSmoothingEnabled = false;
  painter.drawImage(source, 0, 0, out.width, out.height);
  const link = document.createElement('a');
  link.href = URL.createObjectURL(await out.convertToBlob({ type: 'image/png' }));
  link.download = name;
  link.click();
  URL.revokeObjectURL(link.href);
}
