# 0.3.0 (June 10, 2020)

### Breaking changes
- Require [`v2::OutputPins`](https://github.com/rust-embedded/embedded-hal/blob/9e6ab5a1ee8900830bd4fe56f0a84ddb0bccda3f/src/digital/v2.rs)
- [`OutputPin` is now taken by ownership](https://github.com/kellerkindt/w5500/blob/d02bbf7e5cc837e658671d1467305523136376cc/src/lib.rs#L131) instead of mut refs [#13](https://github.com/kellerkindt/w5500/issues/13)

### Changes
- Upgrade to Rust 2018 Edition
- Many doc updates, thanks @jonahbron 
