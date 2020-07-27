# A thing that makes a globe from mapbox tiles

This is just some Rust + WASM thing I'm messing around with.

## Building

This assumes you a working install of rust and wasm-pack.

You'll need a Mapbox Vector Maps API Token, or your own vector tiles server.

### Compile the rust into a node module

```
wasm-pack
cd pkg
```

### Create a symlink to the node package

```
npm link
```

### In the Webpack/JS side, link to the symlink above

```
cd ../globe
npm link papariki
```

### Start the webpack server

Make sure to include your mapbox token here

```
MAPBOX_TOKEN=pk.eyFOOBAR npm run server
```

# Screenshot

![Screenshot](https://user-images.githubusercontent.com/3372/88506717-ac89b380-d02e-11ea-9153-20e3c4e5fcb1.png)
