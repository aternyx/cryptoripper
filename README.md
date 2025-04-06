# vbrute
  Desperate for money, commie?
  Ever saw some sort of "magical crypto miner" videos all over Instagram or any other social media platform? 
  You wish you had that software but cannot pay a price of 500$? 
  Why not use this ~~Python script~~ Rust software.

  **Notice:** it is still under works and can be unstable.

## WARNING / DISCLAIMER
  This program is made for educational purposes. It is only 
  made to demonstrate the glorious security of cryptocurrency. 
  Using this to steal may lead to possible *(severe)* consequences. 
  This should be used for attempts to recover your own wallet that
  you have lost. **I will not be responding to any help related to 
  this script that I recieve on my DMs.** For issues *(NOT HELP)*, open
  a issue in the Issue tab, or if you know how to fix it, do a pull
  request. **Bailing out, you are on your own. Thanks for paying attention.**

# Usage
  Navigate the software as you would usually do in Vim. Type in `:?` for help.
  Import your BLF and BIN files in the program using `:i <bin/blf> <path>`
  Select mode using `:m <seed/pkey/milksad> [12/25]`, for searching using random seedphrase (normal or Milk Sad) or private keys. 
  `[12/25]` is the amount of words for seedphrase, should be used for seed/milksad modes.
  If you want to solve BTC puzzles or search for your wallet in a specified range, use `:rang <start> <end>`, e.g.: `:rang 80000000000000000 fffffffffffffffff`. Valid for btc pkey mode.
  To search on a specific blockchain, use `:bc <btc/eth/bnb/xrp/doge/sol/ltc/bch/bsv>`.
  If you want to output only addresses with balance, use `:cb <true/false | 1/0 | on/off>` or `:chkbal <true/false | 1/0 | on/off>`. By default, this is on, to avoid continuous output of random addresses.
