#!/usr/bin/perl

print "Building in release";
system("cd embedded_firmware && cargo build --release");

print "Copying rust binary...";
system("rust-objcopy -O binary target/thumbv6m-none-eabi/release/embedded_firmware target/thumbv6m-none-eabi/release/embedded_firmware.bin");

print "Preparing port...";
system("stty -F /dev/ttyACM0 ospeed 1200");

print "Flashing program...";
system("~/.arduino15/packages/arduino/tools/bossac/1.7.0-arduino3/bossac -i -d --port=ttyACM0 -U true -i -e -w -v target/thumbv6m-none-eabi/release/embedded_firmware.bin -R");
