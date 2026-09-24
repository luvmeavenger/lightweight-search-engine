@echo off
setlocal

where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust is not installed. Please install Rust from https://rustup.rs and rerun this script.
  exit /b 1
)

if not exist seed_urls.txt (
  > seed_urls.txt echo https://www.rust-lang.org/
  >> seed_urls.txt echo https://doc.rust-lang.org/book/
  >> seed_urls.txt echo https://www.wikipedia.org/
  >> seed_urls.txt echo https://www.example.com/
)

cargo build --release
cargo run -- index search.db seed_urls.txt

echo Setup complete.
echo Run: cargo run -- serve 0.0.0.0:3000
exit /b 0
