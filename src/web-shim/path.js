import { basename as _basename } from "./_files";

export async function basename(p) {
  return _basename(p);
}

export async function dirname(p) {
  if (!p) return "";
  const s = String(p);
  const idx = Math.max(s.lastIndexOf("/"), s.lastIndexOf("\\"));
  return idx === -1 ? "" : s.slice(0, idx);
}

export async function join(...parts) {
  return parts.filter(Boolean).join("/");
}
