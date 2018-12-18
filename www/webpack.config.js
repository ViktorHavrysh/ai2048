const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const dist = path.resolve(__dirname, "dist");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = {
  mode: "production",
  entry: "./src/main.ts",
  devtool: "inline-source-map",
  devServer: {
    contentBase: dist
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "index.html"
    }),

    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "./ai2048-wasm"),
      withTypeScript: true
    })
  ],
  module: {
    rules: [
      { test: /\.worker\.ts/, loader: "worker-loader" },
      { test: /\.ts$/, use: "ts-loader" },
      { test: /\.wasm$/, type: "webassembly/experimental" },
      {
        test: /\.(s*)css$/,
        use: ["style-loader", "css-loader", "sass-loader"]
      },
      { test: /\.ico$/, use: ["file-loader"] },
      {
        test: /\.(woff(2)?|ttf|eot|svg)(\?v=\d+\.\d+\.\d+)?$/,
        use: [
          {
            loader: "file-loader",
            options: {
              outputPath: "fonts/"
            }
          }
        ]
      }
    ]
  },
  resolve: {
    extensions: [".ts", ".js", ".sass", ".wasm", ".ico"],
    modules: ["node_modules"]
  },
  output: {
    path: dist,
    filename: "bundle.js"
  }
};
