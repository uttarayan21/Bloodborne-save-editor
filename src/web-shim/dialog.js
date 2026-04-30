import { fileStore, nextId } from "./_files";

export async function open(_opts = {}) {
  return new Promise((resolve) => {
    const input = document.createElement("input");
    input.type = "file";
    input.style.display = "none";
    input.addEventListener("change", async () => {
      const f = input.files && input.files[0];
      if (!f) {
        resolve(null);
        return;
      }
      const buf = new Uint8Array(await f.arrayBuffer());
      const id = nextId("mem", f.name);
      fileStore.set(id, { name: f.name, bytes: buf });
      resolve(id);
    });
    input.addEventListener("cancel", () => resolve(null));
    document.body.appendChild(input);
    input.click();
    input.remove();
  });
}

export async function save(opts = {}) {
  const def = opts.defaultPath || "save.bin";
  const name = window.prompt("Save as:", def);
  if (!name) return null;
  return nextId("dl", name);
}

export async function message(msg, _opts = {}) {
  window.alert(typeof msg === "string" ? msg : String(msg));
}

export async function confirm(msg, _opts = {}) {
  return window.confirm(typeof msg === "string" ? msg : String(msg));
}

export async function ask(msg, _opts = {}) {
  return window.confirm(typeof msg === "string" ? msg : String(msg));
}
