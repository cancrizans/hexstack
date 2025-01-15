set -e
rm -r build/
cargo build --release --target wasm32-unknown-unknown
mkdir -p build/
mkdir -p build/game/
cp index.html build/game/index.html
cp gfx -r build/game/gfx
cp diags -r build/game/diags
cp audio -r build/game/audio
cp target/wasm32-unknown-unknown/release/hexstack.wasm build/game/hexstack.wasm

cp build/game/hexstack.wasm .

cd build/game

zip -r ../game.zip *
