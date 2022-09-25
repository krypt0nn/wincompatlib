# Wincompatlib

Rust library that contains a set of interfaces to run windows applications on unix-like systems using Wine

## Examples

### Run cmd.exe using system wine

```rust
use wincompatlib::prelude::*;

// Run cmd.exe using system wine
Wine::default().run("cmd");
```

### Print wine version

```rust
use wincompatlib::prelude::*;

// Print wine version
println!("Wine version: {:?}", Wine::default().version().unwrap());
```

### Run cmd.exe using custom wine, and then stop it

```rust
use wincompatlib::prelude::*;

let wine = Wine::from_binary("/path/to/wine");

// Run cmd.exe using custom wine
// and then stop it
wine.run("cmd");
wine.stop_processes(true);
```

### Print DXVK version

```rust
// Requires "dxvk" feature
use wincompatlib::prelude::*;

match Dxvk::get_version("/path/to/prefix") {
    Ok(Some(version)) => println!("DXVK applied: {}", version),
    Ok(None) => println!("DXVK is not applied"),
    Err(err) => eprintln!("Failed to get DXVK version: {}", err)
}
```

### Install DXVK

```rust
// Requires "dxvk" feature
use wincompatlib::prelude::*;

// Same for uninstall_dxvk
let output = Wine::default()
    .install_dxvk("/path/to/setup_dxvk.sh", "/path/to/prefix")
    .expect("Failed to install DXVK");

println!("Installing output: {}", String::from_utf8_lossy(&output.stdout));
```
