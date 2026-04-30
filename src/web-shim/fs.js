// Minimal stubs — the editor only uses fs through the dialog/invoke path
// in web mode, so explicit fs calls (if any) are non-fatal.

export async function readFile(_path) {
  throw new Error("fs.readFile is not available in web mode");
}

export async function writeFile(_path, _data) {
  throw new Error("fs.writeFile is not available in web mode");
}

export async function exists(_path) {
  return false;
}
