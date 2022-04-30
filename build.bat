@echo off

cargo build --all --target wasm32-unknown-unknown --release
xcopy target\wasm32-unknown-unknown\release\kawaii_zoo_nft.wasm .\res\ /y
xcopy target\wasm32-unknown-unknown\release\kawaii_zoo_game.wasm .\res\ /y
