# which

A Rust equivalent of Unix command "which".

## Exmaple

To find wihch rustc exectable binary is using.

``` rust
use which::which;

let result = which::which("rustc").unwrap();
assert_eq!(result, PathBuf::from("/usr/bin/rustc"));

```

## Documentation

The documentation is [available online](http://fangyuanziti.github.io/which-rs/which/).