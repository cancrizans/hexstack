set -e
rm -r build/
cargo build --release --target wasm32-unknown-unknown
mkdir -p build/
mkdir -p build/toko_web/

cp index.html build/toko_web/index.html
cp gfx -r build/toko_web/gfx
cp diags -r build/toko_web/diags
cp audio -r build/toko_web/audio
cp mq_js_bundle -r build/toko_web/mq_js_bundle
cp target/wasm32-unknown-unknown/release/hexstack.wasm build/toko_web/hexstack.wasm
cp build/toko_web/hexstack.wasm .
cd build/toko_web
zip -r ../toko_web.zip *


cd ../..
cargo build --release
mkdir -p build/toko_win/
cp target/release/hexstack.exe build/toko_win/tokonoma.exe
cp gfx -r build/toko_win/gfx
cp diags -r build/toko_win/diags
cp audio -r build/toko_win/audio
cd build/toko_win
zip -r ../toko_win.zip *
