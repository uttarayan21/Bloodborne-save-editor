import { fileStore, basename, downloadBytes } from "./_files";

let wasmReady;

async function getWasm() {
  if (!wasmReady) {
    wasmReady = (async () => {
      const mod = await import("../wasm/pkg/bb_web_wasm.js");
      // wasm-pack target=web entry: default export initializes the module
      if (typeof mod.default === "function") {
        await mod.default();
      }
      return mod;
    })();
  }
  return wasmReady;
}

export async function invoke(cmd, args = {}) {
  const wasm = await getWasm();

  // Translate path-based file IO into in-memory bytes / browser downloads.
  if (cmd === "make_save") {
    const id = args.path;
    const f = fileStore.get(id);
    if (!f) throw new Error("No file selected");
    return wasm.invoke("make_save", { bytes: f.bytes });
  }

  if (cmd === "save") {
    const bytes = wasm.invoke("save", {});
    const name = basename(args.path) || "save.bin";
    downloadBytes(name, bytes);
    return "Changes saved.";
  }

  if (cmd === "export_appearance") {
    const bytes = wasm.invoke("export_appearance", {});
    const name = basename(args.path) || "face.bin";
    downloadBytes(name, bytes);
    return "Successfully exported";
  }

  if (cmd === "import_appearance") {
    const f = fileStore.get(args.path);
    if (!f) throw new Error("No face file selected");
    return wasm.invoke("import_appearance", { bytes: f.bytes });
  }

  return wasm.invoke(cmd, args);
}
