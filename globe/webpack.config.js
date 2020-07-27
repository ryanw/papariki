const path = require('path');
const webpack = require('webpack');

const production = process.env.NODE_ENV == 'production';

module.exports = {
  mode: production ? 'production' : 'development',
  entry: {
    main: './src/index.ts',
  },
  output: {
    filename: '[name].bundle.js',
    path: path.resolve(__dirname, 'dist'),
  },
  devtool: 'source-map',
  devServer: {
    host: '0.0.0.0',
    port: 8088,
    contentBase: path.join(__dirname, 'dist'),
    compress: true,
    disableHostCheck: true,
    historyApiFallback: true,
  },
  module: {
    rules: [
      // Typescript
      {
        test: /\.ts$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: [ '.ts', '.js' ],
  },
  optimization: {
    splitChunks: {
      chunks: 'all',
    },
  },
  plugins:[
    new webpack.DefinePlugin({
      MAPBOX_TOKEN: JSON.stringify(process.env.MAPBOX_TOKEN),
    })
  ],

}
