# ipod_sun
Code execution on the iPod nano 6th and 7th generation

### How?
This tool builds a modified firmware image that abuses two iPod bugs in order to gain code execution:

#### 1) Disk swapping
By swapping the 'disk' and 'osos' sections in a firmware image, the iPod will boot into the standard RetailOS when holding the buttons for disk mode. But, when booting into disk mode the iPod won't verify the 'rsrc' partition as disk mode usually doesn't use it.

#### 2) CVE-2010-1797 (better known as star)
By using a malformed OTF font, we can trigger a stack overflow in CFF parsing. See `src/exploit.rs` for details 

### The result
Custom SCSI command added that can read/write memory and execute arbitrary code.

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

## Supported devices:
- iPod Nano 6th Generation
- iPod Nano 7th Generation (Mid 2015)


# WARNING!
Some devices are not able to boot into DFU, this may be caused by a non-functional battery.

Bad payloads, incorrectly packed firmware and many other causes CAN and HAVE caused permanent bricks.

## Usage
```shell
# Build the patched firmware
cargo r --release -- --device=nano7-refresh

# Flash Firmware-repack.MSE over DFU
```

# Attribution
Base.ttf is one of the payloads from [star](https://github.com/comex/star), used as a CFF template

helpers/viafont/original sourced from [here](http://www.publicdomainfiles.com/show_file.php?id=13949894425072)


# Thanks
q3k for the SCSI handler example and for [wInd3x](https://github.com/freemyipod/wInd3x)

760ceb3b9c0ba4872cadf3ce35a7a494 for [ipodhax](https://github.com/760ceb3b9c0ba4872cadf3ce35a7a494/ipodhax) which inspired a lot of the firmware un/packing code


