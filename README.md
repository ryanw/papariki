# A thing that makes a globe from mapbox tiles

This is just some Rust + WASM thing I'm messing around with.

The entry point for the wasm is src/wasm.rs.

## Building

Build the npm package using `wasm-pack`.

To build a version that runs standalone and serve it with the basic python webserver:

```
wasm-pack build -t web
python -m http.server 8000
```

And open http://localhost:8000/?token=MAPBOX_TOKEN_HERE

# Screenshot

![Screenshot](https://user-images.githubusercontent.com/3372/88788752-e226cc00-d1e9-11ea-9604-294c4cba6543.png)

