const path = require("path");

const HtmlWebpackPlugin = require("html-webpack-plugin");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const TsconfigPathsPlugin = require("tsconfig-paths-webpack-plugin");

const root = path.resolve(__dirname, "app");
const ai2048 = path.resolve(__dirname, "../ai2048-wasm");
const dist = path.resolve(__dirname, "dist");

const appConfig = {
  context: root,
  entry: "main.ts",
  devtool: "cheap-module-source-map",
  devServer: {
    contentBase: dist
  },
  plugins: [new HtmlWebpackPlugin({ template: "index.html" })],
  module: {
    rules: [
      {
        enforce: "pre",
        test: /\.js$/,
        use: "source-map-loader"
      },
      {
        enforce: "pre",
        test: /\.ts$/,
        exclude: /node_modules/,
        use: "tslint-loader"
      },
      {
        test: /\.ts$/,
        exclude: [/node_modules/],
        use: "awesome-typescript-loader"
      },
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
    modules: [root, "node_modules"],
    plugins: [new TsconfigPathsPlugin()]
  },
  output: {
    path: dist,
    filename: "app.js"
  }
};

const workerConfig = {
  entry: "./worker/worker.js",
  target: "webworker",
  plugins: [new WasmPackPlugin({ crateDirectory: ai2048 })],
  resolve: {
    extensions: [".js", ".wasm"]
  },
  output: {
    path: dist,
    filename: "worker.js"
  }
};

module.exports = [appConfig, workerConfig];
