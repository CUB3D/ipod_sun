# There's no good font library for Rust, someone please make one

from fontTools import ttLib
from fontTools import cffLib
import base64
import sys
import io

with open("./in-cff.bin", "rb") as input_font:
    x = cffLib.CFFFontSet()
    x.decompile(input_font, None, isCFF2=False)

    tt = ttLib.TTFont("./helpers/viafont/viafont_converted.otf")

    x[0].charset = tt["CFF "].cff[0].charset

    tt_cff = tt["CFF "].cff[0]
    tt_cff_cs = tt_cff.CharStrings["space"]

    x_cs_glyphs = x[0].getGlyphOrder()
    x_cs_bc = x[0].CharStrings["space"]

    tt_cff.CharStrings["space"] = x[0].CharStrings["space"]
    tt_cff.CharStrings["space"].calcBounds = lambda x:  None

with open("./out-otf.bin", "wb") as out_file:
    tt.save(out_file, False)
