cargo build --example blinky_rtic
rust-objcopy -O binary \
    target/thumbv6m-none-eabi/debug/examples/blinky_rtic \
    target/thumbv6m-none-eabi/debug/examples/blinky_rtic.bin
stty -F /dev/ttyACM0 ospeed 1200
~/.arduino15/packages/arduino/tools/bossac/1.7.0-arduino3/bossac -i -d \
    --port=ttyACM0 -U true -i -e -w -v \
    target/thumbv6m-none-eabi/debug/examples/blinky_rtic.bin -R
