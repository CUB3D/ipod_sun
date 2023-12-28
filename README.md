# ipod_sun
Code execution on the iPod nano 6th and 7th generation

### How?
This tool builds a modified firmware image that abuses two iPod bugs in order to gain code execution:

#### 1) Disk swapping
By swapping the 'disk' and 'osos' sections in a firmware image, the iPod will boot into the standard RetailOS when holding the buttons for disk mode. But, when booting into disk mode the iPod won't verify the 'rsrc' partition as disk mode usually doesn't use it.

#### 2) CVE-2010-1797 (better known as star)
By using a malformed OTF font, we can trigger a stack overflow in CFF parsing. See `src/exploit.rs` for details 

### The result
On the iPod Nano 6th Generation: custom SCSI command added that can dump memory

On the iPod Nano 7th Generation: blind code execution

## Dependencies
For python3:
```
pyfatfs
fonttools
```
Native:
```
arm-none-eabi-gcc
```
Some extra files are needed:
```
helpers/comic.otf -> comic.ttf taken from Windows 10, converted to OTF using fonttools
# Note that any other OTF font that doesn't bootloop with a disk swap should work
# comic.ttf (original) MD5 cb5a21a92d77658df1beee4d51b72777
# helpers/comic.otf MD5 2bc0050ee3171ab80d8aa1b9ee262b48

Firmware-golden.MSE -> MSE file from n6g firmware ipsw, MD5 25bcdf992d580c2c5041d98ce63a9616
Firmware-golden-n7g.MSE -> MSE file from n7g firmware ipsw, MD5 f7dd910f81496d6a703768a08003fabf
```

## Usage (n6g)
```shell
# First build the payload
cd scsi_shellcode
cargo b --release
arm-none-eabi-objcopy -O binary target/thumbv6m-none-eabi/release/scsi_shellcode scsi-stub.bin

# Now build the patched firmware
cargo r --release -- --device=n6g

# Flash Firmware-repack.MSE over DFU
```

## Usage (n7g, very WIP)
# WARNING!
Some devices are not able to boot into DFU, this may be caused by a non-functional battery.

Bad payloads, incorrectly packed firmware and many other causes CAN and HAVE caused permanent bricks.

You have been warned.

We don't have a usable memory read for the n7g, so currently we are limited to blind code execution. No payload is provided. Here be dragons
```shell
# Write a payload and change `src/exploit.rs` to use it

# Build the firmware
cargo r --release -- --device=n7g

# Flash Firmware-repack.MSE over DFU
```

# Attribution
Base.ttf is one of the payloads from [star](https://github.com/comex/star), used as a CFF template

# Thanks
q3k for the SCSI handler example and for [wInd3x](https://github.com/freemyipod/wInd3x)

760ceb3b9c0ba4872cadf3ce35a7a494 for [ipodhax](https://github.com/760ceb3b9c0ba4872cadf3ce35a7a494/ipodhax) which inspired a lot of the firmware un/packing code


