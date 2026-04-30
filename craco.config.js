const path = require("path");

const isWeb = process.env.WEB === "1";

module.exports = {
  webpack: {
    alias: isWeb
      ? {
          "@tauri-apps/api/core": path.resolve(__dirname, "src/web-shim/core.js"),
          "@tauri-apps/api/path": path.resolve(__dirname, "src/web-shim/path.js"),
          "@tauri-apps/plugin-dialog": path.resolve(__dirname, "src/web-shim/dialog.js"),
          "@tauri-apps/plugin-fs": path.resolve(__dirname, "src/web-shim/fs.js"),
          "@tauri-apps/plugin-shell": path.resolve(__dirname, "src/web-shim/shell.js"),
        }
      : {},
    configure: (webpackConfig) => {
      // Allow wasm + async file imports
      webpackConfig.experiments = {
        ...(webpackConfig.experiments || {}),
        asyncWebAssembly: true,
        topLevelAwait: true,
      };
      webpackConfig.module.rules.push({
        test: /\.wasm$/,
        type: "asset/resource",
      });
      return webpackConfig;
    },
  },
};
