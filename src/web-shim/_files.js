// Shared file/blob registry for the web shim.
// Maps virtual "paths" (returned by dialog.open / dialog.save) to file metadata
// so invoke() can pick up the bytes and trigger downloads.

export const fileStore = new Map();
let counter = 0;

export function nextId(prefix, name) {
  counter += 1;
  return `${prefix}://${counter}/${name}`;
}

export function basename(virtualPath) {
  if (!virtualPath) return "";
  return String(virtualPath).split("/").pop().split("\\").pop();
}

export function downloadBytes(name, bytes) {
  const blob = new Blob([bytes], { type: "application/octet-stream" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = name;
  document.body.appendChild(a);
  a.click();
  a.remove();
  URL.revokeObjectURL(url);
}
