import fs
from pyfatfs import PyFatFS
from fontTools import ttLib, subset
import base64
import io
import sys

fat = PyFatFS.PyFatFS("./rsrc.bin", read_only=False)

FONT_BASE = "/Resources/Fonts/Helvetica.ttf"

# Read the file, get its name
b = fat.openbin(FONT_BASE, mode="rb")
original_font = ttLib.TTFont(b)
original_font_name_table = original_font["name"]
b.close()

# Remove the old font
fat.remove(FONT_BASE)

# Open the file to write our font
b = fat.openbin(FONT_BASE, mode="wb")

with open("./in-otf.bin", "rb") as f:
    fake_font = ttLib.TTFont(f)
    fake_font["name"] = original_font_name_table
    fake_font["CFF "].cff[0].CharStrings["space"].calcBounds = lambda x: None

    fake_font.save(b)

b.close()

fat.close()
