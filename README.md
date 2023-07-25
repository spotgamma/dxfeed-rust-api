# dxfeed-rust-api
Rust client for DxFeed API

## Cloning
```sh
git clone --recurse-submodules git@github.com:spotgamma/dxfeed-rust-api.git
```

## Rust Bindgen
[requirements](https://rust-lang.github.io/rust-bindgen/requirements.html)
### On Debian/Ubuntu:
```sh
sudo apt install llvm-dev libclang-dev clang
```

## Pulling latest updates from dxfeed-c-api C-API
```sh
git submodule update --recursive --remote
```
## dxfeed-c-api
See the [dxfeed-c-api](https://github.com/dxFeed/dxfeed-c-api/blob/master/README.md) for the underlying C API types.

## Running
https://github.com/spotgamma/dxfeed-rust-api/blob/a3d4946375a0ddec98b60b97bc7483396a4f4ee8/samples/quote_sub_example/src/main.rs#L65-L134
