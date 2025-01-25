set -e
rm -rf build/

./build_wasm.sh hexstack --release

mkdir -p build/
mkdir -p build/toko_web/


cp index.html build/toko_web/index.html
cp gfx -r build/toko_web/gfx
cp diags -r build/toko_web/diags
cp audio -r build/toko_web/audio
cp mq_js_bundle -r build/toko_web/mq_js_bundle
cp dist/hexstack.js build/toko_web/hexstack.js
cp dist/hexstack_bg.wasm build/toko_web/hexstack_bg.wasm

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
