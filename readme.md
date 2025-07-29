## Rust Proxy Dll Crate



### Description

>  Only support windows

The crate can ease you burden in creating a proxy dll to do dll injection.



### Usage

First, import the crate to your `cdylib` lib

```shell
$ cargo add --git https://github.com/kurikomoe/proxy-dll-rs.git proxy_dll
```

Second, create your own payload dll like this:

```rust
use anyhow::Result;
use windows::Win32::{Foundation::HINSTANCE, System::SystemServices::DLL_PROCESS_ATTACH};

fn payload() -> Result<()> {
    proxy_dll::init(); 
    // Init the proxy_dll, 
    // you can do this at anytime, since the crate ctor takes care of initing.
    // call the init function just to avoid rustc eliminate the code.
    println!("Hello World in Payload");
    Ok(())
}

#[unsafe(export_name = "DllMain")]
extern "system" fn dll_main(_: HINSTANCE, reason: u32, _reserved: isize) -> i32 {
    if reason == DLL_PROCESS_ATTACH {
        payload().ok();
    }
    1
}

```

Finally, build the dll with env vars:

```shell
$ PROXY_DLL_TARGET=version.dll PROXY_DLL_USE_ORIG=false cargo build
// PROXY_DLL_TARGET: the target dll to be proxied, default version.dll
// PROXY_DLL_TARGET_PATH (optional): the target dll path
// PROXY_DLL_USE_ORIG: should the target dll be loaded via ${NAME}_orig.dll

// `ALERT`, you can find the `$name.dll` in your root folder, not under `target/$PROFIEL/`
```



#### Env Vars In details:

Supposed that we have a `test.cpp` like this. `g++ test.cpp -lversion -o test`

```c++
#include <iostream>
#include <cstdint>
#include <windows.h>
int main() {
  auto hdll = LoadLibraryW(L"version.dll");
  LPDWORD lpdwHandle = NULL;
  auto size = GetFileVersionInfoSize("version.dll", NULL);
  uint8_t* buf = new uint8_t[size+1];
  GetFileVersionInfo("version.dll", NULL, size, buf);
  std::cout << "Hello World" << std::endl;
  return 0;
}
```

Then:

```shell
PROXY_DLL_TARGET=version.dll  PROXY_DLL_USE_ORIG=0
test.exe  version.dll
// version.dll is the `proxy dll`, it will load version.dll (same as the LoadLibray load path)

PROXY_DLL_TARGET=version.dll  PROXY_DLL_USE_ORIG=1
test.exe  version.dll version_orig.dll 
// version.dll is the `proxy dll`, it will load the `version_orig.dll` from cwd.
```



### Extra

You can find the demo in `demo_dll` folder.