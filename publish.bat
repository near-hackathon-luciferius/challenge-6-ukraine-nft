@echo off

call near delete kawaii-zoo-game.cryptosketches.testnet cryptosketches.testnet
call near delete kawaii-zoo-donation.cryptosketches.testnet cryptosketches.testnet
call near call kawaii-zoo-nft.cryptosketches.testnet clear_templates --accountId kawaii-zoo-nft.cryptosketches.testnet
call near delete kawaii-zoo-nft.cryptosketches.testnet cryptosketches.testnet

call near create-account kawaii-zoo-game.cryptosketches.testnet --masterAccount cryptosketches.testnet
call near create-account kawaii-zoo-donation.cryptosketches.testnet --masterAccount cryptosketches.testnet
call near create-account kawaii-zoo-nft.cryptosketches.testnet --masterAccount cryptosketches.testnet

call near deploy kawaii-zoo-game.cryptosketches.testnet .\res\kawaii_zoo_game.wasm --initFunction new --initArgs "{}"
call near deploy kawaii-zoo-nft.cryptosketches.testnet .\res\kawaii_zoo_nft.wasm --initFunction new_default_meta --initArgs "{""owner_id"": ""kawaii-zoo-nft.cryptosketches.testnet""}"