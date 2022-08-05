# Rust Chat

## About
This is an attempt to build a web chat solely with Rust and
- [warp](https://github.com/seanmonstar/warp) as a web server framework
- [yew](https://yew.rs/) - a frontend framework for creating web apps using WebAssembly

In current state it is a walking skeleton which can be used to add more functionality using TDD.

A walking skeleton means:
- The chat backend was copied from warp/examples/websocket_chat.rs,
- Its very basic JS frontend was rewrote to yew/wasm
- End-to-end tests with local/remote webdriver were added

So far the most interesting part is the test which starts application (and possibly webdriver/browser locally) and exercises the functionality end-to-end:

https://user-images.githubusercontent.com/25208879/183374770-c5be0b98-be81-4f8c-8f6c-01bb9adf7f23.mp4

## To run tests locally
- Install WebAssembly target
  ```
  rustup target add wasm32-unknown-unknown
  ```
- Install [trunk](https://trunkrs.dev/#install). The SLOWEST way is below, so check the previous link for a better approach
  ```
  cargo install trunk
  ```
- Build:
  ```
  cargo build
  trunk build
  ```
- Depending on your environment you may need to specify which webdriver/browser to use:
  ```
  DEMO_MODE=true cargo test # try to find a webdriver in PATH or using 'which'
  DEMO_MODE=true GECKODRIVER=auto cargo test # use Gecko/Firefox. in case its driver is missing in PATH, the driver will be downloaded
  DEMO_MODE=true GECKODRIVER=/path/to/the/driver cargo test # use Gecko/Firefox with specified driver
  DEMO_MODE=true GECKODRIVER_REMOTE=http://localhost:3030 cargo test # use Gecko/Firefox with remove driver
  # Also supported are SAFARIDRIVER, CHROMEDRIVER, MSEDGEDRIVER - although tested only on Firefox and Chrome
  ```
