# libdxfeed-sys
A light FFI wrapper around the dxfeed-c-api.
This is statically linked

**TODO**:
get compiling on MacOS.  This currently breaks due to hard-coding [`stdc++` here](https://github.com/spotgamma/dxfeed-rust-api/blob/main/libdxfeed-sys/build.rs#L85)
