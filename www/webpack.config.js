const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");
const dist = path.resolve(__dirname, "dist");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

const browserConfig = {
  entry: "./browser/main.ts",
  devtool: "inline-source-map",
  devServer: {
    contentBase: dist
  },
  plugins: [
    new HtmlWebpackPlugin({
      template: "index.html"
    })
  ],
  module: {
    rules: [
      { test: /\.ts$/, use: "ts-loader" },
      {
        test: /\.(s*)css$/,
        use: ["style-loader", "css-loader", "sass-loader"]
      },
      {
        test: /\.ico$/,
        use: [
          {
            loader: "file-loader",
            options: {
              name: "[name].[ext]",
              outputPath: "/"
            }
          }
        ]
      },
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
    extensions: [".ts", ".js", ".sass", ".ico"],
    modules: ["node_modules"]
  },
  output: {
    path: dist,
    filename: "app.js"
  }
};

const workerConfig = {
  entry: "./worker/worker.js",
  target: "webworker",
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "../ai2048-wasm"),
      withTypeScript: true
    })
  ],
  resolve: {
    extensions: [".js", ".wasm"]
  },
  output: {
    path: dist,
    filename: "worker.js"
  }
};

module.exports = [browserConfig, workerConfig];
